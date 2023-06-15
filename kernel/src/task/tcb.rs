use super::context::TaskContext;
use crate::config::{
    task_trap_context_position, task_user_stack_position, PAGE_SIZE, PRIORITY_DIVIDER,
};
use crate::mem::address::*;
use crate::mem::allocator::Frame;
use crate::mem::kernel::{
    kernel_memset_translate, kernel_satp, map_kernel_stack, unmap_kernel_stack,
};
use crate::mem::kernel_stack::{alloc_kstack, kernel_stack_position, KernelStack};
use crate::mem::memory_set::{MapMode::Indirect, MemPermission, MemoryArea};
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use crate::trap::context::TrapContext;
use crate::trap::user_trap_handler;
use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use alloc::string::String;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
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
    pub priority: usize, // 线程优先级: 1~100，优先级大调度越频繁
    pub stride: usize,   // 步长调度
}

impl TaskControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        priority: usize,
        entry_point: usize,
        arg: usize,
    ) -> Self {
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
            priority,
        );
        map_kernel_stack(kstack_bottom, kstack_top, None);
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
        // 传递线程函数参数的地址到a0寄存器
        (*trap_ctx).a[0] = arg;
        return tcb;
    }

    pub fn fork_child_task(
        process: Arc<ProcessControlBlock>,
        parent: Arc<TaskControlBlock>,
        parent_pcb: Arc<ProcessControlBlock>,
    ) -> Self {
        let p_inner = parent.inner.borrow();
        // 分配新的内核栈
        let kstack = alloc_kstack().unwrap();
        let (kstack_bottom, kstack_top) = kernel_stack_position(kstack.0);
        let (p_kstack_bottom, _) = kernel_stack_position(parent.kernel_stack.0);
        let p_memset = &parent_pcb.borrow_inner().memory_set;
        // 映射内核栈并拷贝父进程的数据
        let kstack_data = kernel_memset_translate(VirtAddr(p_kstack_bottom).vpn())
            .unwrap()
            .as_bytes();
        map_kernel_stack(kstack_bottom, kstack_top, Some(kstack_data));
        // 映射用户栈
        let (ustack_bottom, ustack_top) = task_user_stack_position(process.stack_base, parent.tid);
        p_memset
            .areas
            .iter()
            .find(|area| area.start_vpn.base_addr() == ustack_bottom) // 找到父进程的用户栈MemoryArea
            .map(|area| {
                let memset = &mut process.borrow_inner().memory_set;
                let mut frames: BTreeMap<VirtPageNumber, Arc<Frame>> = BTreeMap::new();
                for (vpn, frame) in area.frames.iter() {
                    // 在页表直接将子进程的用户栈映射到父进程的物理页
                    memset.page_table.map(
                        VirtPageNumber(vpn.0),
                        frame.ppn,
                        MemPermission::R.bits() | MemPermission::U.bits(), // 去除写权限，使PageFault触发COW
                    );
                    frames.insert(VirtPageNumber(vpn.0), Arc::clone(frame)); // 增加一个frame的引用
                }
                memset.areas.push(MemoryArea {
                    start_vpn: area.start_vpn,
                    end_vpn: area.end_vpn,
                    mode: Indirect,
                    perm: area.perm,
                    frames: frames,
                });
            });
        // 映射并拷贝trap上下文
        let trap_ctx_vpn = VirtAddr(task_trap_context_position(p_inner.tid)).vpn();
        let trap_ctx_data = p_memset
            .translate(trap_ctx_vpn)
            .unwrap()
            .page_number()
            .as_bytes();
        process.borrow_inner().memory_set.insert_area(
            MemoryArea::new(
                trap_ctx_vpn,
                VirtPageNumber(trap_ctx_vpn.0 + 1),
                Indirect,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            Some(&trap_ctx_data),
        );

        let trap_ctx_ppn = process
            .borrow_inner()
            .memory_set
            .translate(trap_ctx_vpn)
            .unwrap()
            .page_number();
        let inner_tcb = TaskControlBlockInner {
            tid: parent.tid,
            stack: ustack_bottom,
            process: Arc::downgrade(&process),
            task_context: p_inner.task_context.clone(),
            trap_ctx_ppn: trap_ctx_ppn,
            status: p_inner.status,
            exit_code: None,
            priority: p_inner.priority,
            stride: p_inner.stride,
        };
        return Self {
            tid: p_inner.tid,
            kernel_stack: kstack,
            inner: SafeCell::new(inner_tcb),
        };
    }

    // 转换虚拟地址buffer到物理页集合
    pub fn translate_buffer<'a>(&self, addr: usize, len: usize) -> Vec<&'a mut [u8]> {
        let inner = self.inner.borrow();
        let proc = inner.process.upgrade().unwrap();
        proc.translate_buffer(addr, len)
    }

    pub fn translate_string(&self, addr: usize) -> String {
        let inner = self.inner.borrow();
        let proc = inner.process.upgrade().unwrap();
        proc.translate_string(addr)
    }

    // 虚拟地址转换物理地址
    pub fn transalate_virtaddr(&self, va: usize) -> usize {
        let inner = self.inner.borrow();
        let proc = inner.process.upgrade().unwrap();
        proc.translate_va(va)
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

    pub fn wake_up(&self) {
        self.inner.borrow().status = TaskStatus::Ready;
    }

    pub fn increase_stride(&self) {
        let mut inner = self.inner.borrow();
        let pass = PRIORITY_DIVIDER / inner.priority;
        inner.stride += pass;
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
        priority: usize,
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
            priority: priority,
            stride: 0,
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
        let mut inner_pcb = proc.borrow_inner();
        let trap_context = task_trap_context_position(self.tid);
        // 解除栈和trap上下文的映射
        inner_pcb.memory_set.remove_area(VirtAddr(self.stack).vpn());
        inner_pcb
            .memory_set
            .remove_area(VirtAddr(trap_context).vpn());
        drop(inner_pcb);
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
