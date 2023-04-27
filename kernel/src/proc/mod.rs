use crate::arch::riscv::register;
use crate::config::CPUS;
use crate::sync::cell::SafeCell;
use alloc::sync::Arc;

pub mod context;
pub mod loader;
pub mod pcb;
pub mod pid;
pub mod scheduler;

pub fn cpuid() -> usize {
    unsafe { register::read_tp() }
}

pub fn init_processors() {
    // 初始化proc manager（目前仅加载测试程序）
    let mut manager = scheduler::MANAGER.lock();
    let proc1 = Arc::new(pcb::ProcessControlBlock::from_elf_data(
        loader::load_kernel_app("hello_world"),
    ));
    manager.push(proc1);
    let proc2 = Arc::new(pcb::ProcessControlBlock::from_elf_data(
        loader::load_kernel_app("shell"),
    ));
    manager.push(proc2);
    drop(manager);
}
