use crate::devices::pit::get_system_time;
use crate::kernel::cpu::{disable_int_nested, enable_int_nested};

pub extern "C" fn sys_get_system_time() -> u64 {
    get_system_time() as u64
}
