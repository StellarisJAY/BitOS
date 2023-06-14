use crate::sync::cond::Cond;
use crate::sync::mutex::{BlockingMutex, Mutex, SpinMutex};
use crate::task::scheduler::current_proc;
use alloc::sync::Arc;

// mutex create 创建mutex互斥锁
// blocking：是否阻塞
// 返回：锁ID
pub fn mutex_create(blocking: bool) -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();

    let mutex: Option<Arc<dyn Mutex>> = if blocking {
        Some(Arc::new(BlockingMutex::new()))
    } else {
        Some(Arc::new(SpinMutex::new()))
    };
    let result = inner_pcb
        .mutex_table
        .iter()
        .enumerate()
        .find(|(_, item)| (*item).is_some())
        .map(|(i, _)| i);

    if let Some(id) = result {
        inner_pcb.mutex_table[id] = mutex;
        return id as isize;
    } else {
        inner_pcb.mutex_table.push(mutex);
        return inner_pcb.mutex_table.len() as isize - 1;
    }
}

// mutex_lock 互斥锁上锁
// 阻塞锁会导致线程阻塞
pub fn mutex_lock(id: isize) -> isize {
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

pub fn cond_create() -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();

    let cond = Some(Arc::new(Cond::new()));
    let result = inner_pcb
        .cond_table
        .iter()
        .enumerate()
        .find(|(_, item)| (*item).is_some())
        .map(|(i, _)| i);

    if let Some(id) = result {
        inner_pcb.cond_table[id] = cond;
        return id as isize;
    } else {
        inner_pcb.cond_table.push(cond);
        return inner_pcb.cond_table.len() as isize - 1;
    }
}

// wait，cond不存在返回-1，锁不存在返回-2
pub fn cond_wait(id: isize, mutex: isize) -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();

    if let Some(cond) = inner_pcb.cond_table[id as usize].as_ref() {
        match inner_pcb.mutex_table.get(mutex as usize).unwrap() {
            None => return -2,
            Some(mutex) => {
                cond.wait(Arc::clone(&mutex));
                return 0;
            }
        };
    } else {
        return -1;
    }
}

// cond_signal, cond不存在返回-1
pub fn cond_signal(id: isize) -> isize {
    let pcb = current_proc();
    let mut inner_pcb = pcb.borrow_inner();

    if let Some(cond) = inner_pcb.cond_table[id as usize].as_ref() {
        cond.signal();
        return 0;
    } else {
        return -1;
    }
}
