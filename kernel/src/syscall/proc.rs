use crate::proc::pcb::{ProcessControlBlock, ProcessState};
use crate::task::scheduler::{
    add_process, current_task, current_task_trap_context, exit_current_task, find_process,
    push_task, remove_process, schedule_idle,
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
    debug!("child pid: {}", child.pid());
    child.pid()
}

pub fn sys_waitpid(pid: usize) -> isize {
    let task = current_task();
    let current_proc = task.inner.borrow().process.upgrade().unwrap();
    let mut inner = current_proc.borrow_inner();
    let res = inner
        .children
        .iter()
        .enumerate()
        .find(|(_, child)| child.pid() == pid)                // 从当前进程的子进程中找到pid
        .map(|(index, child)| {
            let child_inner = child.borrow_inner();
            if child_inner.status == ProcessState::Zombie {  // 子进程是僵尸进程，删除pcb所有权，回收资源
                remove_process(child.pid());
                return (index, child_inner.exit_code as isize);
            } else {
                return (index, -2);
            }
        });
    if let Some((index, exit_code)) = res {
        if exit_code != -2 {
            inner.children.remove(index);
        }
        return exit_code;
    }
    0
}
