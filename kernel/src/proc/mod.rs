use crate::task::scheduler;
use alloc::sync::Arc;

pub mod loader;
pub mod pcb;
pub mod pid;

pub fn init_processors() {
    pcb::ProcessControlBlock::from_elf_data(loader::load_kernel_app("hello_world"));
    pcb::ProcessControlBlock::from_elf_data(loader::load_kernel_app("thread_test"));
}
