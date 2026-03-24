/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: thread                                                          ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Functions for creating, starting, switching and ending threads. ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Autor:  Michael Schoettner, 15.05.2023                                  ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use crate::consts;
use crate::kernel::coroutines::coroutine::Coroutine;
use crate::kernel::cpu;
use crate::kernel::threads::scheduler;
use crate::kernel::threads::scheduler::get_scheduler;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::naked_asm;
use core::fmt::Display;
use core::sync::atomic::AtomicUsize;
use crate::consts::{STACK_ENTRY_SIZE, STACK_SIZE, USER_STACK_VIRT_END};
use core::{fmt, ptr};
use crate::kernel::paging::pages;
use crate::kernel::paging::pages::PageTable;
use usrlib::user_api::usr_thread_exit;

unsafe extern "C" {
    fn _tss_set_rsp0(rsp0: usize);
}

static THREAD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    THREAD_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

/// Low-level routine for starting a thread.
#[unsafe(naked)]
unsafe extern "C" fn thread_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi", // move stack to subroutine stack pointer
        "call unlock_scheduler",
        // restore all registers
        "popf",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "ret" // jump to kickoff
    )
}

/// Low-level routine for switching to the next thread.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the next coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next thread (the new rsp value).
#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize, next_stack_end: usize, next_pml4: usize) {
    naked_asm!(
        // safe all registers
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "pushf",
        "mov [rdi], rsp", // save rsp to coroutine stack pointer

        // Update TSS rsp0 to 'next_stack_end' (third parameter)
        "mov rdi, rdx", // rdx = next_stack_end
        "call _tss_set_rsp0",

        "mov cr3, rcx", // load new address space (fourth parameter)

        "mov rsp, rsi",   // move stack to next subroutine stack pointer
        "call unlock_scheduler",
        // restore all registers
        "popf",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "ret" // jump to kickoff
    )
}

#[unsafe(naked)]
unsafe extern "C" fn thread_user_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi", // Switch stack
        "pop rsi",
        "pop rdi",
        "iretq" // Return to user mode
    )
}

/// Represents a coroutine in the system.
/// It contains the kernel and user stacks and the entry function.
/// Threads must be registered in the scheduler and are run automatically
/// once the scheduler is started.
#[repr(C)]
pub struct Thread {
    id: usize,
    pid: usize,
    is_kernel_thread: bool,
    kernel_stack: Vec<u64>,
    stack_ptr: usize, // Pointer on the stack to the saved context
    user_stack_top: u64,
    pml4: &'static mut PageTable,
    entry: fn(&[&str]),
    args: Vec<String>,
    name: String,
}

impl Thread {
    /// Create a new thread with the given entry function.
    pub fn new_kernel_thread(entry: fn(&[&str]), args: Vec<String>, name: String) -> Box<Thread> {
        // Allocate memory for the kernel stack and initialize it to zero
        let mut kernel_stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..kernel_stack.capacity() {
            kernel_stack.push(0);
        }

        // Set the stack pointer to the top of the stack
        let stack_ptr = ptr::from_ref(&kernel_stack[kernel_stack.capacity() - 1]) as usize;

        // own address space
        let pml4 = pages::init_kernel_tables();

        // Create a new thread object
        let mut thread = Box::new(Thread {
            id: next_id(),
            pid: 0,
            is_kernel_thread: true,
            kernel_stack,
            stack_ptr,
            user_stack_top: USER_STACK_VIRT_END as u64,
            pml4,
            entry,
            args,
            name,
        });

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();
        thread
    }

    pub fn new_user_thread_old(entry: fn(&[&str]), args: Vec<String>, name: String) -> Box<Thread> {
        let mut thread = Self::new_kernel_thread(entry, args, name);

        thread.is_kernel_thread = false;

        unsafe { pages::map_user_stack(thread.pml4) as *mut u64 };

        thread
    }

    pub fn new_user_thread(app_name: &str, args: Vec<String>, name: String) -> Option<Box<Thread>> {
        let entry: fn(&[&str]) = unsafe { core::mem::transmute(consts::USER_CODE_VIRT_START) };

        let mut thread = Self::new_kernel_thread(entry, args, name);
        thread.is_kernel_thread = false;

        // anwendung laden und mappen
        unsafe {
            if !pages::map_user_app(thread.pml4, app_name) {
                // panic!("App '{}' not found in TAR archive!", app_name);
                return None;
            }
        }

        unsafe { pages::map_user_stack(thread.pml4) as *mut u64 };

        Some(thread)
    }

    pub fn new_user_thread_existing_process(
        entry: fn(&[&str]),
        args: Vec<String>,
        name: String,
        pml4: &'static mut PageTable,
        pid: usize,
        user_stack_top: u64
    ) -> Box<Thread> {
        let mut kernel_stack = Vec::<u64>::with_capacity(consts::STACK_SIZE / 8);
        kernel_stack.resize(kernel_stack.capacity(), 0);

        let stack_ptr = ptr::from_ref(&kernel_stack[kernel_stack.capacity() - 1]) as usize;

        let mut thread = Box::new(Thread {
            id: next_id(),
            pid,
            is_kernel_thread: false,
            kernel_stack,
            stack_ptr,
            pml4,
            entry,
            args,
            name,
            user_stack_top,
        });

        thread.prepare_kernel_stack();
        thread
    }

    /// Start the thread.
    /// This function is only once by the scheduler.
    /// The scheduler does further thread switching via `switch()`.
    pub fn start(&mut self) {
        unsafe {
            pages::write_cr3(self.pml4);
            thread_start(self.stack_ptr);
        }
    }

    /// Switch from the `current` thread to the `next` thread.
    /// This function is called by the scheduler to switch between threads.
    pub unsafe fn switch(current: *mut Thread, next: *mut Thread) {
        // kprintln!("Switching from thread {} to thread {}", (*current).id, (*next).id);
        unsafe {
            let current = &mut *current;
            let next = &*next;
            let next_stack_end = Thread::get_top_of_stack(&next.kernel_stack);
            let next_pml4_ptr = ptr::from_ref(next.pml4) as usize;

            thread_switch(&mut current.stack_ptr, next.stack_ptr, next_stack_end as usize, next_pml4_ptr);
        }
    }

    /// Get the ID of the thread.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Get the name of the thread.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_pid(&self) -> usize {
        self.pid
    }

    pub fn set_pid(&mut self, pid: usize) {
        self.pid = pid;
    }

    /// Prepare the stack of a newly created thread in a way that it can be used
    /// to return to the 'kickoff' function with the thread itself as parameter.
    /// The prepared stack is used in 'thread_start' to start the first thread.
    /// Other threads are started by 'thread_switch' with the prepared stack.
    fn prepare_kernel_stack(&mut self) {
        let kickoff = Thread::kickoff_kernel_thread as u64;
        let thread = ptr::from_mut(self) as u64;
        let length = self.kernel_stack.len();
        let kernel_stack_top = Self::get_top_of_stack(&self.kernel_stack);

        self.kernel_stack[length - 1] = 0x131155; // Dummy return address
        self.kernel_stack[length - 2] = kickoff; // Address of 'kickoff'
        self.kernel_stack[length - 3] = 0; // r8
        self.kernel_stack[length - 4] = 0; // r9
        self.kernel_stack[length - 5] = 0; // r10
        self.kernel_stack[length - 6] = 0; // r11
        self.kernel_stack[length - 7] = 0; // r12
        self.kernel_stack[length - 8] = 0; // r13
        self.kernel_stack[length - 9] = 0; // r14
        self.kernel_stack[length - 10] = 0; // r15
        self.kernel_stack[length - 11] = 0; // rax
        self.kernel_stack[length - 12] = 0; // rbx
        self.kernel_stack[length - 13] = 0; // rcx
        self.kernel_stack[length - 14] = 0; // rdx
        self.kernel_stack[length - 15] = 0; // rsi
        self.kernel_stack[length - 16] = thread; // rdi -> First parameter for 'kickoff'
        self.kernel_stack[length - 17] = 0; // rbp
        self.kernel_stack[length - 18] = 0x2; // rflags (IE = 0); interrupts disabled

        self.stack_ptr = kernel_stack_top as usize - (STACK_ENTRY_SIZE * 18);
    }

    /// Switch this thread from Ring 0 to Ring 3.
    /// For this, the kernel stack is prepared in a way that an 'iretq' instruction
    /// switches to user mode (Ring 3) and the user stack is used. If this function works correctly,
    /// the thread continues in user mode in the function 'kickoff_user_thread'.
    fn switch_to_usermode(&mut self) {
        let mut user_stack_ptr = self.user_stack_top;
        let mut user_str_infos = Vec::with_capacity(self.args.len());

        // copy args to user stack
        for arg in self.args.iter().rev() { // rev() so arg 0 is ontop
            let bytes = arg.as_bytes();
            user_stack_ptr -= bytes.len() as u64;

            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    user_stack_ptr as *mut u8,
                    bytes.len()
                );
            }
            user_str_infos.push((user_stack_ptr, bytes.len()));
        }
        user_str_infos.reverse(); // because .rev()

        // array of &str (ptr, len) (each 16 bytes -> 16-byte aligned)
        user_stack_ptr -= (self.args.len() * 16) as u64;
        user_stack_ptr &= !0xF;
        let array_base = user_stack_ptr;

        for (i, (addr, len)) in user_str_infos.into_iter().enumerate() {
            let entry_ptr = (array_base + (i * 16) as u64) as *mut u64;
            unsafe {
                entry_ptr.offset(0).write_volatile(addr); // .ptr
                entry_ptr.offset(1).write_volatile(len as u64); // .len
            }
        }

        let len = self.kernel_stack.len();
        let kernel_stack_top_addr = Self::get_top_of_stack(&self.kernel_stack) as usize;
        self.kernel_stack[len - 1] = (5 << 3) | 3; // SS (User Data Selector) -> index 5, RPL=3
        self.kernel_stack[len - 2] = user_stack_ptr; // RSP (User Stack Pointer)
        self.kernel_stack[len - 3] = 0x200; // RFLAGS (Bit 9 = Interrupt Flag set)
        self.kernel_stack[len - 4] = (4 << 3) | 3; // CS (User Code Selector) -> index 4, RPL=3
        self.kernel_stack[len - 5] = self.entry as u64;
        self.kernel_stack[len - 6] = array_base;
        self.kernel_stack[len - 7] = self.args.len() as u64;

        let stack_ptr = kernel_stack_top_addr - (7 * STACK_ENTRY_SIZE);
        unsafe {
            thread_user_start(stack_ptr);
        }
    }

    /// Called indirectly by using the prepared stack in 'thread_start' and 'thread_switch'.
    fn kickoff_kernel_thread(&mut self) {
        // Set TSS rsp0 to the top of the kernel stack of this thread
        unsafe {
            let rsp0 = Self::get_top_of_stack(&self.kernel_stack);
            _tss_set_rsp0(rsp0 as usize);
        }

        if self.is_kernel_thread {
            cpu::enable_int(); // interrupts are disabled during thread start
            let arg_refs: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
            ((*self).entry)(&arg_refs);
        } else {
            self.switch_to_usermode();
        }

        get_scheduler().exit();
    }

    /// Called indirectly by using the prepared stack in 'switch_to_usermode'.
    /// At this point, the thread is in user mode (Ring 3) and its entry function is called.
    fn kickoff_user_thread(&self) {
        let arg_refs: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        (self.entry)(&arg_refs);
        usr_thread_exit();
    }

    /// Get a pointer to the top of the given stack.
    fn get_top_of_stack(stack: &Vec<u64>) -> *const u64 {
        unsafe {
            ptr::from_ref(&stack[stack.len() - 1]).offset(1)
        }
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, \'{}\')", self.id, self.name)
    }
}
