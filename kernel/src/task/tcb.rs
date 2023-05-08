use super::context::TaskContext;
use crate::config::{task_trap_context_position, task_user_stack_position, PAGE_SIZE};
use crate::mem::address::*;
use crate::mem::kernel::{map_kernel_stack, unmap_kernel_stack};
use crate::mem::kernel_stack::{alloc_kstack, kernel_stack_position, KernelStack};
use crate::mem::memory_set::{MapMode::Indirect, MemPermission, MemoryArea};
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use alloc::sync::{Arc, Weak};

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exit,
}

// 内核线程控制块
pub struct TaskControlBlock {
    pub kernel_stack: KernelStack, // 内核栈地址
    inner: SafeCell<TaskControlBlockInner>,
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
    pub fn new(process: Arc<ProcessControlBlock>) -> Self {
        let kstask = alloc_kstack().unwrap();
        let (kstack_bottom, kstack_top) = kernel_stack_position(kstask.0);
        let stack_base = process.stack_base;
        let inner = TaskControlBlockInner::new(Arc::clone(&process), stack_base, kstack_bottom);
        map_kernel_stack(kstack_bottom, kstack_top);
        Self {
            kernel_stack: kstask,
            inner: SafeCell::new(inner),
        }
    }
}

impl TaskControlBlockInner {
    // 创建inner线程控制块
    pub fn new(process: Arc<ProcessControlBlock>, stack_base: usize, kernel_stack: usize) -> Self {
        let tid = process.alloc_tid();
        let (stack_bottom, stack_top) = task_user_stack_position(stack_base, tid);
        let trap_context = task_trap_context_position(tid);
        Self::map_memory(
            Arc::clone(&process),
            (stack_bottom, stack_top),
            trap_context,
        );
        let trap_ctx_ppn = process
            .borrow_inner()
            .memory_set
            .vpn_to_ppn(VirtAddr(trap_context).vpn())
            .unwrap();
        Self {
            tid: tid,
            stack: stack_bottom,
            process: Arc::downgrade(&process),
            trap_ctx_ppn: trap_ctx_ppn,
            task_context: TaskContext::switch_ret_context(kernel_stack),
            status: TaskStatus::Ready,
            exit_code: None,
        }
    }

    // 映射线程的栈和trap上下文
    fn map_memory(process: Arc<ProcessControlBlock>, user_stack: (usize, usize), trap_ctx: usize) {
        let mem_set = &mut process.borrow_inner().memory_set;
        // user stack
        mem_set.insert_area(
            MemoryArea::new(
                VirtAddr(user_stack.0).vpn(),
                VirtAddr(user_stack.1).vpn(),
                Indirect,
                MemPermission::R.bits() | MemPermission::W.bits() | MemPermission::U.bits(),
            ),
            None,
        );
        mem_set.insert_area(
            MemoryArea::new(
                VirtAddr(trap_context).vpn(),
                VirtAddr(trap_context + PAGE_SIZE).vpn(),
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
