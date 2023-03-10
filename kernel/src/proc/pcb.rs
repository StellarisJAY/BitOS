use super::pid::{Pid, alloc_pid};
use super::context::ProcessContext;
use alloc::sync::Arc;
use crate::trap::context::TrapContext;
use crate::trap::user_trap_handler;
use crate::mem::memory_set::MemorySet;
use crate::mem::address::*;
use crate::config::*;
use alloc::vec::Vec;

pub struct ProcessControlBlock {
    pid: Pid,
    inner: Arc<InnerPCB>,
}

pub struct InnerPCB {
    mem_size: usize,
    kernel_stack: usize,                          // 内核栈地址
    context: ProcessContext,                      // 进程上下文
    trap_context: *mut TrapContext,               // 陷入上下文
    memory_set: MemorySet,                        // 内存集合
    parent: Option<Arc<ProcessControlBlock>>,     // 父进程pcb
    children: Vec<Arc<ProcessControlBlock>>,      // 子进程pcb集合
}

impl ProcessControlBlock {
    // 从elf数据创建PCB
    pub fn from_elf_data(data: &[u8]) -> Self {
        let pid = alloc_pid().unwrap();
        // 从elf数据创建用户地址空间
        let memset = MemorySet::from_elf_data(data);
        let kernel_satp = crate::mem::kernel::kernel_satp();
        // 映射内核栈
        let (stack_bottom, stack_top) = kernel_stack_position(pid.0);
        crate::mem::kernel::map_kernel_stack(stack_bottom, stack_top);
        let inner = InnerPCB {
            mem_size: data.len(),
            kernel_stack: stack_top,
            context: ProcessContext::switch_ret_context(), // 空的进程上下文，ra指向user_trap_return，使进程被调度后能够回到U模式
            trap_context: &mut TrapContext::user_trap_context(kernel_satp, stack_top, user_trap_handler as usize), // 陷入上下文，记录进程的内核栈信息
            memory_set: memset,
            parent: None,
            children: Vec::new(),
        };
        return Self { pid: pid, inner: Arc::new(inner) }
    }
}



