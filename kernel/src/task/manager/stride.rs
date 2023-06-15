use super::TaskManager;
use crate::proc::pcb::ProcessControlBlock;
use crate::sync::cell::SafeCell;
use crate::task::tcb::{TaskControlBlock, TaskStatus};
use alloc::collections::{BTreeMap, BinaryHeap};
use alloc::sync::Arc;
use alloc::vec::Vec;

// StrideManager 步长调度任务管理器
pub struct StrideManager {
    inner: SafeCell<StrideManagerInner>,
}

struct StrideManagerInner {
    pqueue: BinaryHeap<TCBHolder>, // 小顶堆，堆顶是stride最小的task
    processes: BTreeMap<usize, Arc<ProcessControlBlock>>,
}

// 不能直接为Arc实现Ord和Eq
struct TCBHolder(Arc<TaskControlBlock>);

impl StrideManager {
    pub fn new() -> Self {
        kernel!("using Stride scheduling");
        Self {
            inner: SafeCell::new(StrideManagerInner {
                pqueue: BinaryHeap::new(),
                processes: BTreeMap::new(),
            }),
        }
    }
}

impl TaskManager for StrideManager {
    fn push_task(&self, task: Arc<TaskControlBlock>) {
        let mut inner = self.inner.borrow();
        inner.pqueue.push(TCBHolder(task));
    }

    fn pop_task(&self) -> Option<Arc<TaskControlBlock>> {
        let mut inner = self.inner.borrow();
        let mut not_ready: Vec<TCBHolder> = Vec::new();
        // 找到stride最小且处于READY状态的task
        while let Some(holder) = inner.pqueue.pop() {
            if holder.0.inner.borrow().status == TaskStatus::Ready {
                // 被调度一次，增加stride
                holder.0.increase_stride();
                return Some(holder.0);
            } else {
                not_ready.push(holder);
            }
        }
        // 将非READY的task重新添加到队列
        for holder in not_ready {
            inner.pqueue.push(holder);
        }
        return None;
    }

    fn add_process(&self, proc: Arc<ProcessControlBlock>) {
        self.inner
            .borrow()
            .processes
            .insert(proc.pid(), Arc::clone(&proc));
    }
    fn remove_process(&self, pid: usize) {
        self.inner.borrow().processes.remove(&pid);
    }
}

impl Ord for TCBHolder {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let s1 = self.0.inner.borrow().stride;
        let s2 = other.0.inner.borrow().stride;
        // alloc库中的Heap是大顶堆，需要reverse来实现小顶堆
        return s1.cmp(&s2).reverse();
    }
}

impl PartialOrd for TCBHolder {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TCBHolder {}

impl PartialEq for TCBHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.tid() == other.0.tid()
    }
}
