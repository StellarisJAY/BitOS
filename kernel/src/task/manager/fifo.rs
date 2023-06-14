use super::TaskManager;
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;

// FIFO任务管理器
pub struct FIFOTaskManager {
    inner: SafeCell<FIFOManagerInner>,
}

struct FIFOManagerInner {
    queue: VecDeque<Arc<TaskControlBlock>>, // FIFO队列
    processes: BTreeMap<usize, Arc<ProcessControlBlock>>,
}

impl FIFOTaskManager {
    pub fn new() -> Self {
        return Self {
            inner: SafeCell::new(FIFOManagerInner {
                queue: VecDeque::new(),
                processes: BTreeMap::new(),
            }),
        };
    }
}

impl TaskManager for FIFOTaskManager {
    fn push_task(&self, task: Arc<TaskControlBlock>) {
        let mut inner = self.inner.borrow();
        inner.queue.push_back(Arc::clone(&task));
    }

    fn pop_task(&self) -> Option<Arc<TaskControlBlock>> {
        let mut inner = self.inner.borrow();
        let poped = inner
            .queue
            .iter()
            .enumerate()
            .find(|(i, task)| {
                let inner = task.inner.borrow();
                inner.status == TaskStatus::Ready
            })
            .map(|(i, _)| i);
        if let Some(idx) = poped {
            return Some(inner.queue.swap_remove_front(idx).unwrap());
        } else {
            return None;
        }
    }

    fn add_process(&self, proc: Arc<ProcessControlBlock>) {
        let mut inner = self.inner.borrow();
        inner.processes.insert(proc.pid(), Arc::clone(&proc));
    }

    fn remove_process(&self, pid: usize) {
        let mut inner = self.inner.borrow();
        inner.processes.remove(&pid);
    }
}
