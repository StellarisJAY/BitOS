use super::pid::{Pid, alloc_pid};
use super::context::ProcessContext;
use alloc::sync::Arc;
use crate::trap::context::TrapContext;
use crate::mem::memory_set::MemorySet;
use crate::mem::address::*;

pub struct ProcessControlBlock {
    pid: Pid,
    inner: Arc<InnerPCB>,
}

pub struct InnerPCB {
    mem_size: usize,
    kernel_stack: usize,                         // 内核栈地址
    context: ProcessContext,                     // 进程上下文
    trap_context: *mut TrapContext,              // 陷入上下文
    memory_set: MemorySet,                       // 内存集合
    parent: Option<Arc<ProcessControlBlock>>     // 父进程pcb
}

impl InnerPCB {
    
}


impl ProcessControlBlock {

}



