use super::context::ProcessContext;
use super::pcb::ProcessControlBlock;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use super::loader::load_kernel_app;
use spin::mutex::SpinMutex;
use crate::config::CPUS;

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

lazy_static! {
    pub static ref PROCESSORS: Vec<Arc<Processor>> = Vec::new();
}

// 在当前进程地址空间，转换一个虚拟地址buf
pub fn current_proc_translate_buffer(addr: usize, len: usize) -> Vec<&'static [u8]> {
    let cpuid = super::cpuid();
    let processor = PROCESSORS.get(cpuid).unwrap();
    return processor.current_proc().translate_buffer(addr, len);
}

// 进程管理器添加进程
fn push_process(proc: Arc<ProcessControlBlock>) {
    MANAGER.lock().push(Arc::clone(&proc));
}

// 进程管理器弹出进程
fn pop_process() -> Option<Arc<ProcessControlBlock>> {
    MANAGER.lock().pop()
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


extern "C" {
    // cpu切换进程上下文的汇编函数
    fn __switch(old_ctx: *mut ProcessContext, new_ctx: *const ProcessContext);
}

impl Processor {
    fn current_proc(&self) -> Arc<ProcessControlBlock> {
        return Arc::clone(&self.current_proc);
    }
    // 处理器进入idle状态
    pub fn schedule_idle(&mut self) {
        // idle上下文
        let new_ctx = &self.idle_ctx as *const ProcessContext;
        // 当前进程上下文
        let old_ctx = self.current_proc().context_addr() as *mut ProcessContext;
        unsafe {
            __switch(old_ctx, new_ctx);
        }
        // 当前进程进入FIFO队列
        push_process(Arc::clone(&self.current_proc));
    }

    // 处理器调度下一个进程
    pub fn schedule(&mut self) {

    }
}