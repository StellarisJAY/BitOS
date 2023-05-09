use crate::proc::pcb::ProcessControlBlock;
use crate::task::scheduler::{
    add_process, current_task, current_task_trap_context, exit_current_task, push_task,
    schedule_idle,
};
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> isize {
    exit_current_task(exit_code);
    schedule_idle();
    return 0;
}

pub fn sys_yield() -> isize {
    let task = current_task();
    push_task(task);
    schedule_idle();
    return 0;
}

pub fn sys_fork() -> usize {
    let task = current_task();
    let parent = task.inner.borrow().process.upgrade().unwrap();
    let child = ProcessControlBlock::fork(Arc::clone(&parent), task.tid);
    add_process(Arc::clone(&child));
    child.pid()
}
