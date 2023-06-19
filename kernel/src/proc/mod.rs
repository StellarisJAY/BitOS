use crate::fs::inode::{open_file, OpenFlags};
use crate::task::scheduler;
use alloc::sync::Arc;
pub mod loader;
pub mod pcb;
pub mod pid;

pub fn init_proc() {
    let shell = open_file("/bin/shell", OpenFlags::RDONLY).unwrap();
    let data = shell.read_all();
    pcb::ProcessControlBlock::from_elf_data(data.as_slice());
}
