use crate::proc::scheduler::{exit_current_proc, schedule_idle, push_process, current_proc, current_proc_trap_context};
use crate::proc::pcb::ProcessControlBlock;
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> isize{
    exit_current_proc(exit_code);
    schedule_idle();
    return 0;
}

pub fn sys_yield() -> isize {
    let proc = current_proc();
    push_process(proc);
    schedule_idle();
    return 0;
}

pub fn sys_fork() -> usize {
    let proc = current_proc();
    let child = ProcessControlBlock::fork(Arc::clone(&proc));
    push_process(Arc::clone(&child));
    return child.pid();
}