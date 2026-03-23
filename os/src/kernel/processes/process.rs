use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use crate::kernel::processes::vma::VMA;
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    id: usize,
    name: String,
    pub vmas: Vec<VMA>,
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Process {
            id: pid,
            name: String::from(name),
            vmas: Vec::new(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
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

pub fn is_process_alive(pid: usize) -> bool {
    PROCESSES.lock().contains_key(&pid)
}

