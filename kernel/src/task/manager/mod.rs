use super::tcb::TaskControlBlock;
use crate::proc::pcb::ProcessControlBlock;
use alloc::sync::Arc;

pub mod fifo;
pub mod stride;

// 任务管理器trait
pub trait TaskManager: Send + Sync {
    fn push_task(&self, task: Arc<TaskControlBlock>);
    fn pop_task(&self) -> Option<Arc<TaskControlBlock>>;
    fn add_process(&self, proc: Arc<ProcessControlBlock>);
    fn remove_process(&self, pid: usize);
}
