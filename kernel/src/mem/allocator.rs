use alloc::vec::Vec;
use super::address::*;
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;
use crate::config::{PHYS_MEM_LIMIT};


extern "C" {
    // 内核段结束地址，可分配物理内存的起始地址
    fn ekernel();
}

pub trait MemAllocator {
    fn init(&mut self);
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNumber>;
    fn dealloc(&mut self, ppn: PhysPageNumber);
}

// Frame 物理页帧
pub struct Frame {
    pub ppn: PhysPageNumber
}

// 物理页内存分配器
pub struct VecAllocator {
    end_ppn: usize,
    current_ppn: usize,
    recycled: Vec<usize>,
}

type MemAllocatorImpl = VecAllocator;

lazy_static! {
    pub static ref ALLOCATOR: SpinMutex<MemAllocatorImpl> = SpinMutex::new(MemAllocatorImpl::new());
}

pub fn init() {
    let mut allocator = ALLOCATOR.lock();
    allocator.init();
    drop(allocator);
}

pub fn alloc() -> Option<Frame> {
    let mut allocator = ALLOCATOR.lock();
    let ppn = allocator.alloc();
    drop(allocator);
    return ppn.map(|p|{Frame{ppn: p}});
}

pub fn dealloc(ppn: PhysPageNumber) {
    let mut allocator = ALLOCATOR.lock();
    allocator.dealloc(ppn);
    drop(allocator);
}


impl MemAllocator for VecAllocator {
    fn new() -> Self {
        return Self{current_ppn: 0, end_ppn: 0, recycled: Vec::new()};
    }
    // 初始化物理内存区域，[ekernel,PhysLimit)
    fn init(&mut self) {
        self.current_ppn = PhysAddr(ekernel as usize).page_number().0;
        self.end_ppn = PhysAddr(PHYS_MEM_LIMIT).page_number().0;
    }
    fn alloc(&mut self) -> Option<PhysPageNumber> {
        // 优先分配回收队列中的页
        if !self.recycled.is_empty() {
            return self.recycled.pop().map(|ppn| {
                PhysPageNumber(ppn)
            });
        }else {
            if self.current_ppn == self.end_ppn {
                return None;
            }else {
                let ppn = self.current_ppn;
                self.current_ppn+=1;
                return Some(PhysPageNumber(ppn));
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNumber) {
        self.recycled.push(ppn.0);
    }
}

// Drop 自动回收物理页
impl Drop for Frame {
    fn drop(&mut self) {
        // 清空物理页的数据
        self.ppn.as_bytes().fill(0);
        dealloc(self.ppn);
    }
}