use super::context::ProcessContext;
use super::pid::{alloc_pid, Pid};
use crate::config::*;
use crate::mem::address::*;
use crate::mem::memory_set::MemorySet;
use crate::sync::cell::SafeCell;
use crate::task::tid::TidAllocator;
use crate::trap::context::TrapContext;
use crate::trap::user_trap_handler;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefMut;

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
    pub stack_base: usize,
    inner: SafeCell<InnerPCB>,
}

pub struct InnerPCB {
    pub mem_size: usize,
    pub kernel_stack: usize,                      // 内核栈地址
    pub context: ProcessContext,                  // 进程上下文
    pub trap_context: PhysPageNumber,             // 陷入上下文
    pub memory_set: MemorySet,                    // 内存集合
    pub parent: Option<Arc<ProcessControlBlock>>, // 父进程pcb
    pub children: Vec<Arc<ProcessControlBlock>>,  // 子进程pcb集合
    pub status: ProcessState,
    pub exit_code: i32,
    pub tid_allocator: TidAllocator,
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
        let (memset, entry_point, user_stack_sp, stack_base) = MemorySet::from_elf_data(data);
        let kernel_satp = crate::mem::kernel::kernel_satp();
        // 映射内核栈
        let (stack_bottom, stack_top) = kernel_stack_position(pid.0);
        crate::mem::kernel::map_kernel_stack(stack_bottom, stack_top);

        let trap_context_ppn = memset.vpn_to_ppn(VirtAddr(TRAP_CONTEXT).vpn()).unwrap();
        let mut inner = InnerPCB {
            mem_size: data.len(),
            kernel_stack: stack_top,
            context: ProcessContext::switch_ret_context(stack_top), // 空的进程上下文，ra指向user_trap_return，使进程被调度后能够回到U模式
            trap_context: trap_context_ppn,
            memory_set: memset,
            parent: None,
            children: Vec::new(),
            status: ProcessState::Ready,
            exit_code: 0,
            tid_allocator: TidAllocator::new(0, 128),
        };
        // 创建trap ctx
        let trap_ctx = inner.get_trap_context();
        (*trap_ctx) = TrapContext::user_trap_context(
            kernel_satp,
            stack_top,
            user_trap_handler as usize,
            entry_point,
            user_stack_sp,
        );
        return Self {
            pid: pid,
            stack_base: stack_base,
            inner: SafeCell::new(inner),
        };
    }

    // fork 子进程
    pub fn fork(parent: Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {
        let pid = alloc_pid().unwrap();
        let mut p_inner = parent.borrow_inner();
        let (memset, user_stack_top) = MemorySet::from_parent(&p_inner.memory_set);
        let kernel_satp = crate::mem::kernel::kernel_satp();
        // 映射内核栈
        let (stack_bottom, stack_top) = kernel_stack_position(pid.0);
        crate::mem::kernel::map_kernel_stack(stack_bottom, stack_top);
        let trap_context_ppn = memset.vpn_to_ppn(VirtAddr(TRAP_CONTEXT).vpn()).unwrap();
        let mut inner = InnerPCB {
            mem_size: p_inner.mem_size,
            kernel_stack: stack_top,
            context: ProcessContext::switch_ret_context(stack_top), // 空的进程上下文，ra指向user_trap_return，使进程被调度后能够回到U模式
            trap_context: trap_context_ppn,
            memory_set: memset,
            parent: Some(Arc::clone(&parent)),
            children: Vec::new(),
            status: ProcessState::Ready,
            exit_code: 0,
            tid_allocator: TidAllocator::new(0, 128), // todo 子进程的线程分配器
        };
        let pcb = Arc::new(ProcessControlBlock {
            pid: pid,
            inner: SafeCell::new(inner),
        });
        p_inner.children.push(Arc::clone(&pcb));
        p_inner.memory_set.remove_write_permission();
        drop(p_inner);
        return pcb;
    }

    pub fn copy_on_write(&self, vpn: VirtPageNumber) -> bool {
        let mut inner = self.inner.borrow();
        let vpn_valid = inner.memory_set.copy_on_write(vpn);
        return vpn_valid;
    }

    pub fn user_satp(&self) -> usize {
        return self.inner.borrow().memory_set.satp();
    }
    // 转换虚拟地址buffer到物理页集合
    pub fn translate_buffer(&self, addr: usize, len: usize) -> Vec<&'static mut [u8]> {
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

    pub fn trap_context(&self) -> &'static mut TrapContext {
        unsafe {
            let inner = self.inner.borrow();
            let ctx = inner.trap_context.base_addr() as usize as *mut TrapContext;
            drop(inner);
            return ctx.as_mut().unwrap();
        }
    }

    pub fn trap_context_addr(&self) -> usize {
        return self.inner.borrow().trap_context.base_addr();
    }

    pub fn borrow_inner(&self) -> RefMut<'_, InnerPCB> {
        self.inner.borrow()
    }

    pub fn pid(&self) -> usize {
        return self.pid.0;
    }

    pub fn alloc_tid(&self) -> usize {
        self.borrow_inner().tid_allocator.alloc().unwrap()
    }

    pub fn dealloc_tid(&self, tid: usize) {
        self.borrow_inner().tid_allocator.dealloc(tid);
    }
}
