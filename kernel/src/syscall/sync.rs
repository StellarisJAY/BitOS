use crate::task::scheduler::current_proc;
use crate::sync::mutex::{Mutex, BlockingMutex, SpinMutex};
use alloc::sync::Arc;

// mutex create 创建mutex互斥锁
// blocking：是否阻塞
// 返回：锁ID
pub fn mutex_create(blocking: bool) -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();
    
    let mutex: Option<Arc<dyn Mutex>> = if blocking {
        Some(Arc::new(BlockingMutex::new()))
    }else {
        Some(Arc::new(SpinMutex::new()))
    };
    
    let id = inner_pcb.mutex_table.iter().enumerate()
    .find(|(_, item)| item.is_none())
    .map(|(i, _)| i)
    .unwrap();
    inner_pcb.mutex_table[id] = mutex;
    return id as isize;
}

// mutex_lock 互斥锁上锁
// 阻塞锁会导致线程阻塞
pub fn mutex_lock(id: isize) -> isize{
    let pcb = current_proc();
    let inner_pcb = pcb.borrow_inner();
    
    let mutex = Arc::clone(inner_pcb.mutex_table[id as usize].as_ref().unwrap());
    drop(inner_pcb);
    drop(pcb);
    mutex.lock();
    0
}

// mutex_unlock 互斥锁释放
pub fn mutex_unlock(id: isize) -> isize {
    let pcb = current_proc();
    let inner_pcb = pcb.borrow_inner();

    let mutex = Arc::clone(inner_pcb.mutex_table[id as usize].as_ref().unwrap());
    drop(inner_pcb);
    drop(pcb);
    mutex.unlock();
    0
}