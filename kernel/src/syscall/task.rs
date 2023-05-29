use crate::task::scheduler::{current_proc, push_task};
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;

pub fn create_thread(entry: usize, arg: usize) -> usize {
    let pcb = current_proc();
    let task = Arc::new(TaskControlBlock::new(Arc::clone(&pcb), entry, arg));
    let tid = task.tid();
    pcb.borrow_inner().tasks.push(Arc::clone(&task));
    push_task(task);
    return tid;
}

pub fn wait_tid(tid: usize) -> isize {
    let pcb = current_proc();
    let tasks = &pcb.borrow_inner().tasks;
    for tcb in tasks {
        if tcb.tid() == tid {
            let inner = tcb.inner.borrow();
            if inner.status == TaskStatus::Exit {
                match inner.exit_code {
                    None => return 0,
                    Some(code) => return code,
                }
            } else {
                return -2;
            }
        }
    }
    return -3;
}
