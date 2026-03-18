use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::AtomicUsize;
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    id: usize,
    name: String
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Process { id: pid, name: String::from(name) }
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
