use crate::devices::pcspk::{aerodynamic, tetris};

pub extern "C" fn sys_play_tetris() {
    tetris();
    aerodynamic();
}
