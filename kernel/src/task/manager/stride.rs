use super::TaskManager;
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;

// StrideManager 步长调度任务管理器
pub struct StrideManager {
    inner: SafeCell<StrideManagerInner>,
}

struct StrideManagerInner {
    processes: BTreeMap<usize, Arc<ProcessControlBlock>>,
}

impl StrideManager {
    pub fn new() -> Self {
        Self {
            inner: SafeCell::new(StrideManagerInner {
                processes: BTreeMap::new(),
            }),
        }
    }
}

impl TaskManager for StrideManager {
    fn push_task(&self, task: Arc<TaskControlBlock>) {
        panic!("unimplemented")
    }
    fn pop_task(&self) -> Option<Arc<TaskControlBlock>> {
        panic!("unimplemented")
    }
    fn add_process(&self, proc: Arc<ProcessControlBlock>) {
        panic!("unimplemented")
    }
    fn remove_process(&self, pid: usize) {
        panic!("unimplemented")
    }
}
