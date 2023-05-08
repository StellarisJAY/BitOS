use super::context::TaskContext;
use crate::config::{task_trap_context_position, task_user_stack_position, PAGE_SIZE};
use crate::mem::address::*;
use crate::mem::kernel::{kernel_satp, map_kernel_stack, unmap_kernel_stack};
use crate::mem::kernel_stack::{alloc_kstack, kernel_stack_position, KernelStack};
use crate::mem::memory_set::{MapMode::Indirect, MemPermission, MemoryArea};
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use crate::trap::context::TrapContext;
use crate::trap::user_trap_handler;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exit,
}

// 内核线程控制块
pub struct TaskControlBlock {
    pub tid: usize,
    pub kernel_stack: KernelStack, // 内核栈地址
    pub inner: SafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub tid: usize,                         // 线程id
    pub stack: usize,                       // 线程用户栈底地址
    pub process: Weak<ProcessControlBlock>, // 进程控制块引用
    pub trap_ctx_ppn: PhysPageNumber,       // trap上下文
    pub task_context: TaskContext,          // 线程上下文
    pub status: TaskStatus,
    pub exit_code: Option<isize>,
}

impl TaskControlBlock {
    pub fn new(process: Arc<ProcessControlBlock>, entry_point: usize) -> Self {
        let tid = process.alloc_tid();
        let kstask = alloc_kstack().unwrap();
        let (kstack_bottom, kstack_top) = kernel_stack_position(kstask.0);
        let stack_base = process.stack_base;
        let (ustack_bottom, ustack_top) = task_user_stack_position(stack_base, tid);
        let inner = TaskControlBlockInner::new(
            Arc::clone(&process),
            tid,
            ustack_bottom,
            ustack_top,
            kstack_top,
        );
        map_kernel_stack(kstack_bottom, kstack_top);
        let tcb = Self {
            tid: tid,
            kernel_stack: kstask,
            inner: SafeCell::new(inner),
        };
        let trap_ctx = tcb.trap_context();
        *trap_ctx = TrapContext::user_trap_context(
            kernel_satp(),
            kstack_top,
            user_trap_handler as usize,
            entry_point,
            ustack_top,
        );
        return tcb;
    }

    // 转换虚拟地址buffer到物理页集合
    pub fn translate_buffer<'a>(&self, addr: usize, len: usize) -> Vec<&'a mut [u8]> {
        let inner = self.inner.borrow();
        let proc = inner.process.upgrade().unwrap();
        proc.translate_buffer(addr, len)
    }
    // 获取进程上下文的虚拟地址
    pub fn context_addr(&self) -> usize {
        let mut inner = self.inner.borrow();
        let addr = &mut inner.task_context as *mut TaskContext as usize;
        drop(inner);
        return addr;
    }

    pub fn trap_context<'a>(&self) -> &'a mut TrapContext {
        unsafe {
            let inner = self.inner.borrow();
            let ctx = inner.trap_ctx_ppn.base_addr() as usize as *mut TrapContext;
            drop(inner);
            return ctx.as_mut().unwrap();
        }
    }

    pub fn trap_context_addr(&self) -> usize {
        return self.inner.borrow().trap_ctx_ppn.base_addr();
    }

    pub fn user_satp(&self) -> usize {
        self.inner.borrow().process.upgrade().unwrap().user_satp()
    }

    pub fn tid(&self) -> usize {
        self.tid
    }
}

impl TaskControlBlockInner {
    // 创建inner线程控制块
    pub fn new(
        process: Arc<ProcessControlBlock>,
        tid: usize,
        ustack_bottom: usize,
        ustack_top: usize,
        kstack_sp: usize,
    ) -> Self {
        let trap_context = task_trap_context_position(tid);
        Self::map_memory(
            Arc::clone(&process),
            ustack_bottom,
            ustack_top,
            trap_context,
        );
        let trap_ctx_ppn = process
            .borrow_inner()
            .memory_set
            .vpn_to_ppn(VirtAddr(trap_context).vpn())
            .unwrap();
        Self {
            tid: tid,
            stack: ustack_bottom,
            process: Arc::downgrade(&process),
            trap_ctx_ppn: trap_ctx_ppn,
            task_context: TaskContext::switch_ret_context(kstack_sp),
            status: TaskStatus::Ready,
            exit_code: None,
        }
    }

    // 映射线程的栈和trap上下文
    fn map_memory(
        process: Arc<ProcessControlBlock>,
        ustack_bottom: usize,
        ustack_top: usize,
        trap_ctx: usize,
    ) {
        let mem_set = &mut process.borrow_inner().memory_set;
        // user stack
        mem_set.insert_area(
            MemoryArea::new(
                VirtAddr(ustack_bottom).vpn(),
                VirtAddr(ustack_top).vpn(),
                Indirect,
                MemPermission::R.bits() | MemPermission::W.bits() | MemPermission::U.bits(),
            ),
            None,
        );
        mem_set.insert_area(
            MemoryArea::new(
                VirtAddr(trap_ctx).vpn(),
                VirtAddr(trap_ctx + PAGE_SIZE).vpn(),
                Indirect,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            None,
        );
    }
    // 回收线程资源
    fn dealloc_reasource(&self) {
        let proc = self.process.upgrade().unwrap();
        let mem_set = &mut proc.borrow_inner().memory_set;
        let trap_context = task_trap_context_position(self.tid);
        // 解除栈和trap上下文的映射
        mem_set.remove_area(VirtAddr(self.stack).vpn());
        mem_set.remove_area(VirtAddr(trap_context).vpn());
        // 回收tid
        proc.dealloc_tid(self.tid);
    }
}

impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        let (bottom, _) = kernel_stack_position(self.kernel_stack.0);
        unmap_kernel_stack(bottom);
    }
}

impl Drop for TaskControlBlockInner {
    fn drop(&mut self) {
        self.dealloc_reasource();
    }
}
