use super::pid::{alloc_pid, Pid};
use crate::config::*;
use crate::mem::address::*;
use crate::mem::memory_set::MemorySet;
use crate::sync::cell::SafeCell;
use crate::task::scheduler::{add_process, push_task};
use crate::task::tcb::TaskControlBlock;
use crate::task::tid::TidAllocator;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::vec;
use core::cell::RefMut;
use crate::sync::mutex::Mutex;
use crate::sync::cond::Cond;
use crate::fs::File;
use crate::fs::stdio::{Stdin, Stdout};

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
    pub memory_set: MemorySet,                    // 内存集合
    pub parent: Option<Arc<ProcessControlBlock>>, // 父进程pcb
    pub children: Vec<Arc<ProcessControlBlock>>,  // 子进程pcb集合
    pub status: ProcessState,
    pub exit_code: i32,
    pub tid_allocator: TidAllocator,
    pub tasks: Vec<Arc<TaskControlBlock>>,
    pub mutex_table: Vec<Option<Arc<dyn Mutex>>>, // 进程持有的mutex表，option表示一个mutex槽位是否空闲
    pub cond_table: Vec<Option<Arc<Cond>>>,
    pub fd_table: Vec<Option<Arc<dyn File>>>,     // 进程持有的fd表
}

impl ProcessControlBlock {
    // 从elf数据创建PCB
    pub fn from_elf_data(data: &[u8]) -> Arc<ProcessControlBlock> {
        let pid = alloc_pid().unwrap();
        // 从elf数据创建用户地址空间
        let (memset, entry_point, stack_base) = MemorySet::from_elf_data(data);
        let kernel_satp = crate::mem::kernel::kernel_satp();

        let trap_context_ppn = memset.vpn_to_ppn(VirtAddr(TRAP_CONTEXT).vpn()).unwrap();
        let mut inner = InnerPCB {
            mem_size: data.len(),
            memory_set: memset,
            parent: None,
            children: Vec::new(),
            status: ProcessState::Ready,
            exit_code: 0,
            tid_allocator: TidAllocator::new(0, MAX_THREADS),
            tasks: Vec::with_capacity(MAX_THREADS),
            mutex_table: Vec::new(),
            cond_table: Vec::new(),
            fd_table: vec![
                Some(Arc::new(Stdin{})),    // fd=0, stdin
                Some(Arc::new(Stdout{})),   // fd=1, stdout
                Some(Arc::new(Stdout{})),   // fd=2, stderr -> stdout
            ],
        };
        let proc = Arc::new(Self {
            pid: pid,
            stack_base: stack_base,
            inner: SafeCell::new(inner),
        });
        // 创建main线程，然后将main线程交给调度器
        let task = Arc::new(TaskControlBlock::new(Arc::clone(&proc), entry_point, 0));
        push_task(Arc::clone(&task));
        proc.inner.borrow().tasks.push(task);
        let res = Arc::clone(&proc);
        add_process(proc);
        return res;
    }

    // fork 子进程
    pub fn fork(parent: Arc<ProcessControlBlock>, tid: usize) -> Arc<ProcessControlBlock> {
        let pid = alloc_pid().unwrap();
        let mut p_inner = parent.borrow_inner();
        let memset = MemorySet::from_parent(&p_inner.memory_set, parent.stack_base);
        let kernel_satp = crate::mem::kernel::kernel_satp();

        // 拷贝父进程的fd
        let mut fd_table: Vec<Option<Arc<dyn File>>> = Vec::new();
        for item in p_inner.fd_table.iter() {
            if let Some(fd) = item {
                fd_table.push(Some(Arc::clone(fd)));
            }else {
                fd_table.push(None);
            }
        }

        let mut inner = InnerPCB {
            mem_size: p_inner.mem_size,
            memory_set: memset,
            parent: Some(Arc::clone(&parent)),
            children: Vec::new(),
            status: ProcessState::Ready,
            exit_code: 0,
            tid_allocator: p_inner.tid_allocator.clone(),
            tasks: Vec::new(),
            mutex_table: Vec::new(),
            cond_table: Vec::new(),
            fd_table: fd_table,
        };
        let pcb = Arc::new(ProcessControlBlock {
            stack_base: parent.stack_base,
            pid: pid,
            inner: SafeCell::new(inner),
        });
        p_inner.children.push(Arc::clone(&pcb));
        p_inner.memory_set.remove_write_permission();

        // 获取调用fork的线程tcb
        let mut parent_task: Option<Arc<TaskControlBlock>> = None;
        for task in p_inner.tasks.iter() {
            if task.tid() == tid {
                parent_task = Some(Arc::clone(task));
                break;
            }
        }
        // fork_child_task之前必须释放父进程的inner
        drop(p_inner);
        // 复制当前线程，将该线程加入子进程
        let child_task = Arc::new(TaskControlBlock::fork_child_task(
            Arc::clone(&pcb),
            parent_task.unwrap(),
            Arc::clone(&parent),
        ));
        pcb.borrow_inner().tasks.push(Arc::clone(&child_task));
        // 提交线程给调度器
        push_task(Arc::clone(&child_task));
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

    pub fn translate_va(&self, va: usize) -> usize {
        let inner = self.inner.borrow();
        inner.memory_set.va_to_pa(VirtAddr(va)).unwrap().0
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

impl Drop for ProcessControlBlock {
    fn drop(&mut self) {
        debug!("process {} dropped", self.pid.0);
    }
}
