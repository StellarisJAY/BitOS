use crate::arch::riscv::register;
use alloc::sync::Arc;
use crate::config::CPUS;
use crate::sync::cell::SafeCell;

pub mod context;
pub mod pcb;
pub mod pid;
pub mod scheduler;
pub mod loader;

pub fn cpuid() -> usize {
    unsafe {
        register::read_tp()
    }
}

pub fn init_processors() {
    // 初始化proc manager（目前仅加载测试程序）
    let mut manager = scheduler::MANAGER.lock();
    let proc = Arc::new(pcb::ProcessControlBlock::from_elf_data(loader::load_kernel_app("hello_world")));
    manager.push(proc);
    drop(manager);
}