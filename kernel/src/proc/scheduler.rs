use super::context::ProcessContext;
use super::pcb::{ProcessControlBlock, ProcessState};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use super::loader::load_kernel_app;
use spin::mutex::SpinMutex;
use crate::config::CPUS;
use crate::trap::context::TrapContext;
use crate::sync::cell::SafeCell;
use super::cpuid;

// FIFO进程管理器
pub struct ProcManager {
    queue: VecDeque<Arc<ProcessControlBlock>> // FIFO队列
}

// 处理器，负责调度运行一个进程
pub struct Processor {
    idle_ctx: ProcessContext,
    current_proc: Option<Arc<ProcessControlBlock>>,
}

// 从内核.data段加载init_proc的elf数据
lazy_static! {
    pub static ref INIT_PROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::from_elf_data(load_kernel_app("init_proc")));
}

lazy_static! {
    pub static ref MANAGER: SpinMutex<ProcManager> = SpinMutex::new(ProcManager::new());
}

lazy_static! {
    pub static ref PROCESSORS: Vec<SafeCell<Processor>> = {
        let mut v: Vec<SafeCell<Processor>> = Vec::new();
        for i in 0..CPUS {
            v.push(SafeCell::new(Processor::new()));
        }
        return v;
    };
}

// 处理器调度循环
pub fn schedule() {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    // 如果没有可用进程，处理器在该循环空转
    loop {
        let mut p = processor.borrow();
        if let Some(pcb) = pop_process() {
            let new_ctx = pcb.context_addr() as *const ProcessContext;
            let old_ctx = p.idle_context_ptr();
            p.current_proc = Some(pcb);
            drop(p);
            // switch函数调用之后，idle_ctx的ra将保存schedule循环的pc
            // 下一次schedule_idle后恢复ra并ret，会回到schedule循环中
            unsafe {
                __switch(old_ctx, new_ctx);
            }
        }
    }
}

pub fn schedule_idle() {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    let mut p = processor.borrow();
    // idle上下文
    let new_ctx = &p.idle_ctx as *const ProcessContext;
    let current_proc = p.current_proc().unwrap();
    // 当前进程上下文
    let old_ctx = current_proc.context_addr() as *mut ProcessContext;
    p.current_proc = None;
    drop(p);
    unsafe {
        __switch(old_ctx, new_ctx);
    }
}

// 在当前进程地址空间，转换一个虚拟地址buf
pub fn current_proc_translate_buffer(addr: usize, len: usize) -> Vec<&'static [u8]> {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    let p = processor.borrow();
    let buf = p.current_proc().unwrap().translate_buffer(addr, len);
    return buf;
}

pub fn current_proc_trap_context() -> &'static mut TrapContext {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    return processor.borrow().current_proc().unwrap().trap_context();
}

pub fn current_proc_trap_addr() -> usize {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    return processor.borrow().current_proc().unwrap().trap_context_addr();
}

pub fn current_proc_satp() -> usize {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    return processor.borrow().current_proc().unwrap().user_satp();
}

pub fn exit_current_proc(exit_code: i32) {
    let mut processor = PROCESSORS.get(cpuid()).unwrap().borrow();
    let current_proc = processor.current_proc().unwrap();
    let mut inner = current_proc.borrow_inner();
    // 设置进程退出码和状态
    inner.exit_code = exit_code;
    inner.status = ProcessState::Exit;
    // 将子进程的parent设置为init_proc
    for child in inner.children.iter() {
        let mut child_inner = child.borrow_inner();
        child_inner.parent = Some(Arc::clone(&INIT_PROC));
        drop(child_inner);
    }
    drop(inner);
    drop(current_proc);
    drop(processor);
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
    pub fn new() -> Self {
        Self { idle_ctx: ProcessContext::empty(), current_proc: None }
    }

    fn current_proc(&self) -> Option<Arc<ProcessControlBlock>> {
        match &self.current_proc {
            Some(proc) => return Some(Arc::clone(&proc)),
            None=>None
        }
    }
    // 获取idle进程的ctx
    pub fn idle_context_ptr(&mut self) -> *mut ProcessContext {
        let ctx = &mut self.idle_ctx;
        return ctx as *mut ProcessContext;
    }
}