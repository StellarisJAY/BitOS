use crate::task::scheduler;
use alloc::sync::Arc;
use crate::fs::inode::{open_file, OpenFlags};
pub mod loader;
pub mod pcb;
pub mod pid;

pub fn init_processors() {
    let shell = open_file("shell", OpenFlags::RDONLY).unwrap();
    let data = shell.read_all();
    pcb::ProcessControlBlock::from_elf_data(data.as_slice());
}
