use crate::proc::pcb::ProcessControlBlock;
use crate::proc::scheduler::{
    current_proc, current_proc_trap_context, exit_current_proc, push_process, schedule_idle,
};
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> isize {
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
