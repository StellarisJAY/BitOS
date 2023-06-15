use crate::task::scheduler::{current_proc, push_task};
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;

pub fn create_thread(entry: usize, arg: usize) -> usize {
    let pcb = current_proc();
    let task = Arc::new(TaskControlBlock::new(Arc::clone(&pcb), 10, entry, arg));
    let tid = task.tid();
    pcb.borrow_inner().tasks.push(Arc::clone(&task));
    push_task(task);
    return tid;
}

pub fn wait_tid(tid: usize) -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();
    let mut idx = 0;
    let mut code = -1;
    for tcb in inner_pcb.tasks.iter() {
        if tcb.tid() == tid {
            let inner = tcb.inner.borrow();
            if inner.status == TaskStatus::Exit {
                code = match inner.exit_code {
                    None => return 0,
                    Some(code) => return code,
                }
            } else {
                code = -2;
            }
            break;
        }
        idx += 1;
    }
    // waittid结束后删除线程引用
    if code != -2 && code != -1 {
        inner_pcb.tasks.swap_remove(idx);
    }
    return code;
}
