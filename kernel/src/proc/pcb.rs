use super::pid::{Pid, alloc_pid};
use super::context::ProcessContext;
use alloc::sync::Arc;
use crate::trap::context::TrapContext;
use crate::trap::user_trap_handler;
use crate::mem::memory_set::MemorySet;
use crate::mem::address::*;
use crate::config::*;
use alloc::vec::Vec;
use crate::sync::cell::SafeCell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Exit,
    Zombie,
}

pub struct ProcessControlBlock {
    pid: Pid,
    inner: SafeCell<InnerPCB>,
}

pub struct InnerPCB {
    mem_size: usize,
    kernel_stack: usize,                          // 内核栈地址
    context: ProcessContext,                      // 进程上下文
    trap_context: PhysPageNumber,                 // 陷入上下文
    memory_set: MemorySet,                        // 内存集合
    parent: Option<Arc<ProcessControlBlock>>,     // 父进程pcb
    children: Vec<Arc<ProcessControlBlock>>,      // 子进程pcb集合
    status: ProcessState,
    exit_code: i32,
}

impl InnerPCB {
    pub fn get_trap_context(&mut self) -> &mut TrapContext {
        unsafe {
            let ptr = self.trap_context.base_addr() as *mut TrapContext;
            ptr.as_mut().unwrap()
        }
    }
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

        let trap_context_ppn = memset.vpn_to_ppn(VirtAddr(TRAP_CONTEXT).vpn()).unwrap();
        let mut inner = InnerPCB {
            mem_size: data.len(),
            kernel_stack: stack_top,
            context: ProcessContext::switch_ret_context(), // 空的进程上下文，ra指向user_trap_return，使进程被调度后能够回到U模式
            trap_context: trap_context_ppn,
            memory_set: memset,
            parent: None,
            children: Vec::new(),
            status: ProcessState::Ready,
            exit_code: 0,
        };
        // 创建trap ctx
        let trap_ctx = inner.get_trap_context();
        (*trap_ctx) = TrapContext::user_trap_context(kernel_satp, stack_top, user_trap_handler as usize);
        return Self { pid: pid, inner: SafeCell::new(inner) }
    }

    pub fn user_satp(&self) -> usize {
        return self.inner.borrow().memory_set.satp();
    }
    // 转换虚拟地址buffer到物理页集合
    pub fn translate_buffer(&self, addr: usize, len: usize) -> Vec<&'static [u8]> {
        let inner = self.inner.borrow();
        let res = inner.memory_set.translate_buffer(addr, len);
        drop(inner);
        return res;
    }
    // 获取进程上下文的虚拟地址
    pub fn context_addr(&self) -> usize {
        let mut inner = self.inner.borrow();
        let addr = &mut inner.context as *mut ProcessContext as usize;
        drop(inner);
        return addr;
    }

    pub fn trap_context(&self) -> *const TrapContext {
        let inner = self.inner.borrow();
        let ctx = inner.trap_context.base_addr() as usize as *const TrapContext;
        drop(inner);
        return ctx;
    }
}



