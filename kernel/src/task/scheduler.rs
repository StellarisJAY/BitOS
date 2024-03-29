use super::context::TaskContext;
use super::manager::fifo::FIFOTaskManager;
use super::manager::stride::StrideManager;
use super::manager::TaskManager;
use super::tcb::{TaskControlBlock, TaskStatus};
use crate::arch::riscv::register::read_tp;
use crate::config::{task_trap_context_position, ManagerType, CPUS, TASK_MANAGER};
use crate::proc::pcb::{ProcessControlBlock, ProcessState};
use crate::proc::pid::Pid;
use crate::sync::cell::SafeCell;
use crate::trap::context::TrapContext;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;

// 处理器，负责调度运行一个任务
pub struct Processor {
    idle_ctx: TaskContext,
    current_task: Option<Arc<TaskControlBlock>>,
}

lazy_static! {
    pub static ref MANAGER: SpinMutex<Arc<dyn TaskManager>> = {
        let manager: Arc<dyn TaskManager>;
        // 可选不同类型的调度器
        match TASK_MANAGER {
            ManagerType::FIFO => manager = Arc::new(FIFOTaskManager::new()),
            ManagerType::STRIDE => manager = Arc::new(StrideManager::new()),
        }
        SpinMutex::new(manager)
    };
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

pub fn cpuid() -> usize {
    unsafe { read_tp() }
}

// 处理器调度循环
pub fn schedule() {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    // 如果没有可用任务，处理器在该循环空转
    loop {
        let mut p = processor.borrow();
        if let Some(tcb) = pop_task() {
            let new_ctx = tcb.context_addr() as *const TaskContext;
            let old_ctx = p.idle_context_ptr();
            p.current_task = Some(tcb);
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
    let new_ctx = &p.idle_ctx as *const TaskContext;
    let task = p.current_task().unwrap();
    // 当前任务上下文
    let old_ctx = task.context_addr() as *mut TaskContext;
    p.current_task = None;
    drop(p);
    unsafe {
        __switch(old_ctx, new_ctx);
    }
}

pub fn exit_current_task(exit_code: i32) {
    let task = current_task();
    let mut inner = task.inner.borrow();
    inner.exit_code = Some(exit_code as isize);
    inner.status = TaskStatus::Exit;
    // 是main线程，退出进程
    if inner.tid == 0 {
        let proc = inner.process.upgrade().unwrap();
        let mut inner_pcb = proc.borrow_inner();
        inner_pcb.exit_code = exit_code;
        inner_pcb.tasks.clear();
        // 子进程变成僵尸进程，等待父进程wait回收资源
        if let Some(parent) = &inner_pcb.parent {
            inner_pcb.status = ProcessState::Zombie;
        } else {
            inner_pcb.status = ProcessState::Exit;
            // 没有父进程，删除PCB的所有权，回收资源
            remove_process(proc.pid());
        }
        inner_pcb.children.iter_mut().for_each(|child| {
            child.borrow_inner().parent = None;
        });
        drop(inner_pcb);
        drop(proc);
    }
    drop(inner);
    drop(task);
}

pub fn block_current_task() {
    let task = current_task();
    let mut task_inner = task.inner.borrow();
    task_inner.status = TaskStatus::Blocked;
    drop(task_inner);
    push_task(task);
    schedule_idle();
}

pub fn yield_current_task() {
    let task = current_task();
    push_task(task);
    schedule_idle();
}

pub fn current_task_translate_buffer<'a>(addr: usize, len: usize) -> Vec<&'a mut [u8]> {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    let p = processor.borrow();
    let buf = p.current_task().unwrap().translate_buffer(addr, len);
    return buf;
}

pub fn current_task_translate_string(addr: usize) -> String {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    let p = processor.borrow();
    p.current_task().unwrap().translate_string(addr)
}

pub fn current_task_trap_context<'a>() -> &'a mut TrapContext {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    return processor.borrow().current_task().unwrap().trap_context();
}

pub fn current_task_trap_va() -> usize {
    let tid = current_task().tid;
    task_trap_context_position(tid)
}

pub fn current_task_satp() -> usize {
    let processor = PROCESSORS.get(cpuid()).unwrap();
    return processor.borrow().current_task().unwrap().user_satp();
}

pub fn current_tid() -> usize {
    current_task().tid
}

pub fn current_task() -> Arc<TaskControlBlock> {
    let mut processor = PROCESSORS.get(cpuid()).unwrap().borrow();
    let task = processor.current_task().unwrap();
    return Arc::clone(&task);
}

pub fn current_proc() -> Arc<ProcessControlBlock> {
    let task = current_task();
    let inner_task = task.inner.borrow();
    inner_task.process.upgrade().unwrap()
}

pub fn push_task(task: Arc<TaskControlBlock>) {
    MANAGER.lock().push_task(task);
}

pub fn pop_task() -> Option<Arc<TaskControlBlock>> {
    MANAGER.lock().pop_task()
}

pub fn add_process(process: Arc<ProcessControlBlock>) {
    MANAGER.lock().add_process(process);
}

pub fn remove_process(pid: usize) {
    MANAGER.lock().remove_process(pid);
}

extern "C" {
    // cpu切换任务上下文的汇编函数
    fn __switch(old_ctx: *mut TaskContext, new_ctx: *const TaskContext);
}

impl Processor {
    pub fn new() -> Self {
        Self {
            idle_ctx: TaskContext::empty(),
            current_task: None,
        }
    }

    fn current_task(&self) -> Option<Arc<TaskControlBlock>> {
        match &self.current_task {
            Some(task) => return Some(Arc::clone(&task)),
            None => None,
        }
    }
    // 获取idle任务的ctx
    pub fn idle_context_ptr(&mut self) -> *mut TaskContext {
        let ctx = &mut self.idle_ctx;
        return ctx as *mut TaskContext;
    }
}
