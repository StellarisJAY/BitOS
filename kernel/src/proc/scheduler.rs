use super::context::ProcessContext;
use super::pcb::ProcessControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use super::loader::load_kernel_app;
use spin::mutex::SpinMutex;

// FIFO进程管理器
pub struct ProcManager {
    queue: VecDeque<Arc<ProcessControlBlock>> // FIFO队列
}

// 处理器，负责调度运行一个进程
pub struct Processor {
    idle_ctx: ProcessContext,
    current_proc: Arc<ProcessControlBlock>,
}

// 从内核.data段加载init_proc的elf数据
lazy_static! {
    pub static ref INIT_PROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::from_elf_data(load_kernel_app("init_proc")));
}

lazy_static! {
    pub static ref MANAGER: SpinMutex<ProcManager> = SpinMutex::new(ProcManager::new());
}

impl ProcManager {
    pub fn new() -> Self {
        return Self { queue: VecDeque::new()};
    }

    pub fn push(&mut self, process: Arc<ProcessControlBlock>) {
        self.queue.push_back(Arc::clone(&process));
    }

    pub fn pop(&mut self) -> Option<Arc<ProcessControlBlock>> {
        return self.queue.pop_front();
    }
}