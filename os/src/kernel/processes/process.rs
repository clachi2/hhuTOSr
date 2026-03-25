use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use x86_64::VirtAddr;
use usrlib::term_println;
use crate::consts::USER_STACK_VIRT_START;
use crate::kernel::processes::vma::{VmaType, VMA};
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    id: usize,
    name: String,
    pub vmas: Vec<VMA>,
    pub thread_count: usize,
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Process {
            id: pid,
            name: String::from(name),
            vmas: Vec::new(),
            thread_count: 1,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}

pub fn next_thread_index(pid: usize) -> usize {
    let mut map = PROCESSES.lock();
    if let Some(process) = map.get_mut(&pid) {
        let idx = process.thread_count;
        process.thread_count += 1;
        idx
    } else {
        0
    }
}

pub fn process_count() -> usize {
    PROCESSES.lock().len()
}

pub fn add_process(process: Process) {
    /*
     * Hier muss Code eingefuegt werden
     */
    let mut map = PROCESSES.lock();
    map.insert(process.id, process);
}

pub fn remove_process(process_id: usize) {
    /*
     * Hier muss Code eingefuegt werden
     */
    let mut map = PROCESSES.lock();
    map.remove(&process_id);

}

pub fn get_app_name(process_id: usize) -> Option<String> {
    /*
     * Hier muss Code eingefuegt werden
     */
    let map = PROCESSES.lock();
    map.get(&process_id).map(|p| p.name.clone())
}

pub fn address_is_stack_vma(process_id: usize, virt_addr: u64) -> bool{
    let map = PROCESSES.lock();
    if let Some(process) = map.get(&process_id) {
        for vma in &process.vmas {
            if vma.get_type() == VmaType::Stack && vma.is_inside(virt_addr){
                return true;
            }
        }
    }
    false
}


pub fn add_vma(process_id: usize, vma: VMA) -> Result<(), &'static str> {
    /*
     * Hier muss Code eingefuegt werden
     */

    let mut map = PROCESSES.lock();
    let process = map.get_mut(&process_id).ok_or("Process not found")?;

    for existing_vma in &process.vmas {
        if existing_vma.overlaps(&vma) {
            return Err("VMA overlaps with existing region");
        }
    }

    process.vmas.push(vma);
    Ok(())
}

pub fn dump_process_vmas(process_id: usize) {
    let map = PROCESSES.lock();
    if let Some(process) = map.get(&process_id) {
        kprintln!("VMAs for process '{}' (PID {}):", process.name, process.id);
        for vma in &process.vmas {
            kprintln!("  {:?}", vma);
        }
    } else {
        kprintln!("process with PID {} not found", process_id);
    }
}

pub fn dump_all_process_vmas() {
    let map = PROCESSES.lock();
    for (pid, process) in map.iter().map(|(pid, process)| (*pid, process)) {
        kprintln!("VMAs for process '{}' (PID {}):", process.name, pid);
        term_println!("VMAs for process '{}' (PID {}):", process.name, pid);
        for vma in &process.vmas {
            kprintln!("  {:?}", vma);
            term_println!("  {:?}", vma);
        }
    }
}

pub fn is_process_alive(pid: usize) -> bool {
    PROCESSES.lock().contains_key(&pid)
}

