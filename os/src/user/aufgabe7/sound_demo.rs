use alloc::string::String;
use crate::devices::pcspk;
use crate::devices::pcspk::{tetris, aerodynamic};

pub fn sound_demo(args: &[String]) {
    tetris();
    aerodynamic();
}
