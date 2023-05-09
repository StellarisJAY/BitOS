use crate::task::scheduler::{current_task, push_task};
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;

pub fn create_thread(entry: usize) -> usize {
    let cur_task = current_task();
    let inner_tcb = cur_task.inner.borrow();
    let pcb = inner_tcb.process.upgrade().unwrap();
    let task = Arc::new(TaskControlBlock::new(Arc::clone(&pcb), entry));
    let tid = task.tid();
    pcb.borrow_inner().tasks.push(Arc::clone(&task));
    push_task(task);
    return tid;
}

pub fn wait_tid(tid: usize) -> isize {
    let cur_task = current_task();
    let inner_tcb = cur_task.inner.borrow();
    let pcb = inner_tcb.process.upgrade().unwrap();
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
