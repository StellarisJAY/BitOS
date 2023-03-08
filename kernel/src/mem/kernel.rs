use super::address::*;
use super::memory_set::{MapMode, MemPermission, MemoryArea, MemorySet};
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;

// 内核内存集合
lazy_static! {
    pub static ref KERNEL_MEMSET: SpinMutex<MemorySet> = SpinMutex::new(MemorySet::new());
}

extern "C" {
    fn stext(); fn etext();
    fn srodata(); fn erodata();
    fn sdata(); fn edata();
    fn sbss(); fn ebss();
}

impl MemorySet {
    // 初始化内核，将内核内存直接映射到页表
    pub fn init_kernel() {
        let mut memory_set = KERNEL_MEMSET.lock();
        // 内核.text段，R|X
        memory_set.insert_area(MemoryArea::new(
                VirtAddr(stext as usize).vpn(),
                VirtAddr(etext as usize).vpn(),
                MapMode::Direct,
                MemPermission::X.bits() | MemPermission::R.bits()), None);
        kernel!("kernlel .text section mapped, vpn range: [{},{}]", VirtAddr(stext as usize).vpn().0, VirtAddr(etext as usize).vpn().0);
        // 内核.rodata段，只读
        memory_set.insert_area(MemoryArea::new(
                VirtAddr(srodata as usize).vpn(),
                VirtAddr(erodata as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits()), None);
        // 内核.data段，可读可写
        memory_set.insert_area(MemoryArea::new(
                VirtAddr(sdata as usize).vpn(),
                VirtAddr(edata as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits()), None);
        // 内核.bss段，可读可写
        memory_set.insert_area(MemoryArea::new(
                VirtAddr(sbss as usize).vpn(),
                VirtAddr(ebss as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits()), None);
        drop(memory_set);
    }
}

#[allow(unused)]
pub fn kernel_map_test() {
    let memory_set = KERNEL_MEMSET.lock();
    let text = VirtAddr(stext as usize).vpn();
    debug!("text range: {}, {}", stext as usize, etext as usize);
    debug!("text vpn: {}", text.0);
    let pte = memory_set.translate(text).unwrap();
    assert!(!pte.is_writable(), "should not be writable");
    debug!("ppn: {}, vpn: {}", pte.page_number().0, text.0);
    assert!(pte.page_number().0 == text.0, "should be direct map");
    drop(memory_set);

    kernel!("kernel memory map test passed!");
}