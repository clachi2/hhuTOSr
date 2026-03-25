/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: scheduler                                                       ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: A basic round-robin scheduler for cooperative threads.          ║
   ║         No priorities supported.                                        ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Autor:  Michael Schoettner, 15.05.2023                                  ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use crate::consts::{PAGE_SIZE, USER_CODE_VIRT_START, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::cpu;
use crate::kernel::multiboot::MULTIBOOT_INFO;
use crate::kernel::paging::pages;
use crate::kernel::paging::pages::PageTable;
use crate::kernel::processes::process::{add_process, add_vma, remove_process, Process};
use crate::kernel::processes::vma::{VmaType, VMA};
use crate::kernel::threads::idle_thread::idle_thread;
use crate::kernel::threads::thread;
use crate::kernel::threads::thread::Thread;
use crate::library::queue::LinkedQueue;
use crate::library::utils::strings_equal;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Display;
use core::sync::atomic::{AtomicBool, AtomicUsize};
use core::{fmt, ptr};
use spin::{Mutex, Once};
use usrlib::allocator::is_locked;

/// Global scheduler instance
static SCHEDULER: Once<Scheduler> = Once::new();

pub static SCHEDULER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Global access to the scheduler.
pub fn get_scheduler() -> &'static Scheduler {
    SCHEDULER.call_once(|| Scheduler::new())
}

/// Unlock the scheduler state.
/// This function is called from assembly code.
/// Usually, the mutex would be unlocked automatically when going out of scope.
/// However, since we switch to a different thread in `yield_cpu()` and `exit()`,
/// the scope is not left and the mutex remains locked.
/// As a workaround, we provide this function to unlock the scheduler manually.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlock_scheduler() {
    unsafe {
        get_scheduler().state.force_unlock();
    }
}

/// The state of the scheduler.
/// It contains the active thread and the ready queue with all other threads.
/// The state is contained in its own struct so that it can be locked via a mutex.
struct SchedulerState {
    active_thread: Option<Box<Thread>>,
    ready_queue: LinkedQueue<Box<Thread>>,
    alive_threads: Vec<(usize, usize, String)>,
    zombie: Option<Box<Thread>>, // thread that has exited but is not freed yet (next thread will free it)
    zombie_pml4: Option<*mut PageTable>,
}

unsafe impl Send for SchedulerState {}

/// Represents the scheduler.
/// It is round-robin-based and uses a queue to manage the threads.
pub struct Scheduler {
    state: Mutex<SchedulerState>,
}

impl Scheduler {
    /// Create a new scheduler instance with an empty ready queue
    /// and an idle thread as the active thread.
    pub fn new() -> Self {
        let state = SchedulerState {
            active_thread: Some(Thread::new_kernel_thread(
                idle_thread,
                Vec::new(),
                String::from("idle"),
            )),
            ready_queue: LinkedQueue::new(),
            alive_threads: Vec::new(),
            zombie: None,
            zombie_pml4: None,
        };

        Scheduler {
            state: Mutex::new(state),
        }
    }

    /// Get the ID of the currently active thread.
    pub fn get_active_tid(&self) -> usize {
        let state = self.state.lock();

        state.active_thread.as_ref().unwrap().get_id()
    }

    /// Get the PID of the currently active thread.
    pub fn get_active_pid(&self) -> usize {
        let state = self.state.lock();
        state.active_thread.as_ref().unwrap().get_pid()
    }

    /// Start the scheduler.
    /// This function must only be called once.
    pub fn schedule(&self) {
        let mut state = self.state.lock();
        SCHEDULER_ACTIVE.store(true, core::sync::atomic::Ordering::Relaxed);

        // The active thread is never None, since we must at least have the idle thread.
        state.active_thread.as_mut().unwrap().start();
    }

    /// Register a new thread in the ready queue.
    pub fn ready(&self, thread: Box<Thread>) {
        let mut state = self.state.lock();
        state
            .alive_threads
            .push((thread.get_id(), thread.get_pid(), thread.get_name()));
        state.ready_queue.enqueue(thread);
    }

    /// Re-register a thread that was blocked in the ready queue.
    pub fn ready_after_block(&self, thread: Box<Thread>) {
        let mut state = self.state.lock();
        state.ready_queue.enqueue(thread);
    }

    /// Terminate the current (calling) thread and switch to the next one.
    pub fn exit(&self) {
        let mut state = self.state.lock();

        // free zombie if exits
        state.zombie.take(); // drops Box<Thread> → frees kernel_stack Vec
        if let Some(pml4) = state.zombie_pml4.take() {
            unsafe {
                pages::free_user_page_table(pml4);
            }
        }

        let current = state.active_thread.take().unwrap();
        state
            .alive_threads
            .retain(|&(id, pid, ref name)| id != current.get_id());

        let pid = current.get_pid();
        let is_last_of_process = if pid != 0 {
            let threads_with_same_pid = state
                .alive_threads
                .iter()
                .filter(|&&(id, pidx, _)| pid == pidx)
                .count();
            threads_with_same_pid == 0
        } else {
            false
        };

        if is_last_of_process {
            remove_process(pid);
            state.zombie_pml4 = Some(current.get_pml4_ptr()); // free after context switch
        } else if pid == 0 {
            state.zombie_pml4 = Some(current.get_pml4_ptr()); // kernel thread free page table frames
        }

        state.zombie = Some(current); // free after context switch

        // The idle thread never exits, so there must be at least one thread in the queue.
        let next = state.ready_queue.dequeue().unwrap();

        // Set the dequeued thread as the active thread,
        // overwriting the current one, which we want to exit.
        state.active_thread = Some(next);

        unsafe {
            // Switch to the next thread.
            // `current` still contains the old thread we want to exit,
            // while `state.active_thread` contains the next one.
            Thread::switch(
                state.zombie.as_mut().unwrap().as_mut(),
                state.active_thread.as_mut().unwrap().as_mut(),
            );
        }
    }

    /// Yield the CPU and switch to the next thread in the ready queue.
    pub fn yield_cpu(&self) {
        if let Some(mut state) = self.state.try_lock() {
            if !SCHEDULER_ACTIVE.load(core::sync::atomic::Ordering::Relaxed) {
                return; // Do not yield if the scheduler is not initialized
            }
            if is_locked() {
                return; // Do not yield if the allocator is locked
            }

            // free zombie if exits
            state.zombie.take();
            if let Some(pml4) = state.zombie_pml4.take() {
                unsafe {
                    pages::free_user_page_table(pml4);
                }
            }

            let mut current = state.active_thread.take().unwrap();
            let current_ptr = current.as_mut() as *mut Thread;
            if let Some(dequeued) = state.ready_queue.dequeue() {
                state.ready_queue.enqueue(current);
                state.active_thread = Some(dequeued);
                unsafe {
                    Thread::switch(current_ptr, state.active_thread.as_mut().unwrap().as_mut());
                }
            } else {
                state.active_thread = Some(current);
            }
        }
    }

    /// Kill the thread with the given ID by removing it from the ready queue.
    pub fn kill(&self, to_kill_id: usize) {
        // check if killing active thread
        {
            let state = self.state.lock();
            if let Some(active) = &state.active_thread {
                if active.get_id() == to_kill_id {
                    drop(state);
                    self.exit();
                    return;
                }
            }
        }

        let mut state = self.state.lock();

        // Remove from ready queue, taking ownership so we can inspect before drop.
        let killed = state
            .ready_queue
            .remove_and_return(|t| t.get_id() == to_kill_id);

        state
            .alive_threads
            .retain(|&(id, _, ref _name)| id != to_kill_id);

        if let Some(thread) = killed {
            let pid = thread.get_pid();

            let is_last = pid != 0
                && state
                    .alive_threads
                    .iter()
                    .filter(|&&(_, p, _)| p == pid)
                    .count()
                    == 0;

            if is_last {
                let pml4_ptr = thread.get_pml4_ptr();
                drop(thread);
                drop(state);
                remove_process(pid);
                unsafe {
                    pages::free_user_page_table(pml4_ptr);
                }
            } else {
                // not last thread of process
                drop(thread);
            }
        }
    }

    /// Check if the scheduler state is currently locked.
    pub fn is_locked(&self) -> bool {
        self.state.is_locked()
    }

    /// Prepare the current thread for blocking.
    /// This functions disables interrupts and return the current thread,
    /// as well as the return value from `cpu::disable_int_nested()`.
    /// To complete the blocking operation call `switch_from_blocked_thread()`,
    /// which will enable interrupts again and resume the scheduler.
    pub fn prepare_block(&self) -> (Box<Thread>, bool) {
        let int = cpu::disable_int_nested();
        let mut state = self.state.lock();

        let current = state.active_thread.take().unwrap();

        (current, int)
    }

    /// Complete a blocking operation begun with `prepare_block()`.
    /// This resumes the scheduler and switches to the next thread in the ready queue.
    pub unsafe fn switch_from_blocked_thread(
        &self,
        blocked_thread: *mut Thread,
        interrupts_enabled: bool,
    ) {
        let mut state = self.state.lock();

        if let Some(next) = state.ready_queue.dequeue() {
            state.active_thread = Some(next);

            unsafe {
                Thread::switch(
                    &mut *blocked_thread,
                    state.active_thread.as_mut().unwrap().as_mut(),
                );
            }
            cpu::enable_int_nested(interrupts_enabled);
        } else {
            unsafe {
                state.active_thread = Some(Box::from_raw(blocked_thread));
            }
            drop(state);
            cpu::enable_int_nested(interrupts_enabled);
            self.yield_cpu();
        }
    }

    // active wait
    pub fn wait_on_thread(&self, id: usize) {
        loop {
            if let Some(state) = self.state.try_lock() {
                if !state.alive_threads.iter().any(|&(tid, _, _)| tid == id) {
                    return;
                }
            }
            get_scheduler().yield_cpu();
        }
    }

    pub fn thread_count(&self) -> usize {
        self.state.lock().alive_threads.len()
    }

    pub fn to_string(&self) -> String {
        let processes_string = self
            .state
            .lock()
            .alive_threads
            .iter()
            .map(|(id, pid, name)| format!("   tid:{}, pid:{}, name:{}", id, pid, name))
            .collect::<Vec<String>>()
            .join("\n");

        format!("processes:\n{}", processes_string)
    }

    /// Legt einen neuen Prozess an und startet ihn in einem User-Thread.
    pub fn spawn_process(&self, app_name: &str, args_str: &str) -> u64 {
        let mb_info = MULTIBOOT_INFO.get().expect("Multiboot info missing");
        let archive = mb_info.get_initrd_archive().expect("Initrd missing");

        let mut app_size = None;
        for file in archive.entries() {
            if let Ok(name) = file.filename().as_str() {
                if strings_equal(name, app_name) {
                    app_size = Some(file.data().len() as u64);
                    break;
                }
            }
        }
        let Some(app_size) = app_size else {
            return 0;
        };

        let process = Process::new(app_name);
        let pid = process.get_id();
        add_process(process);

        let args: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
        let aligned_app_size = (app_size + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);

        let code_vma = VMA::new(
            USER_CODE_VIRT_START as u64,
            USER_CODE_VIRT_START as u64 + aligned_app_size,
            VmaType::Code,
        );
        let stack_vma = VMA::new(
            USER_STACK_VIRT_START as u64,
            USER_STACK_VIRT_END as u64,
            VmaType::Stack,
        );
        if add_vma(pid, code_vma).is_err() || add_vma(pid, stack_vma).is_err() {
            remove_process(pid);
            return 0;
        }

        let Some(mut unwrapped_thread) =
            Thread::new_user_thread(app_name, args, String::from(app_name))
        else {
            remove_process(pid);
            return 0;
        };

        unwrapped_thread.set_pid(pid);
        self.ready(unwrapped_thread);
        pid as u64
    }
}

impl Display for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state = self.state.lock();
        let active = state.active_thread.as_ref().unwrap();

        write!(f, "active: {}, ready: {}", active, state.ready_queue)
    }
}
