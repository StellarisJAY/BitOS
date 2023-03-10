use super::address::*;
use super::memory_set::{MapMode, MemPermission, MemoryArea, MemorySet};
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;
use alloc::vec;

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

pub fn kernel_satp() -> usize {
    KERNEL_MEMSET.lock().satp()
}

impl MemorySet {
    // 初始化内核地址空间，将内核内存直接映射到页表
    pub fn init_kernel() {
        let mut memory_set = KERNEL_MEMSET.lock();
        // 内核.text段，R|X
        memory_set.insert_area(MemoryArea::new(
                VirtAddr(stext as usize).vpn(),
                VirtAddr(etext as usize).vpn(),
                MapMode::Direct,
                MemPermission::X.bits() | MemPermission::R.bits()), None);
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

    let in_rodata = (srodata as usize + erodata as usize) / 2;
    let in_text = (stext as usize + etext as usize) / 2;
    let in_data = (sdata as usize + edata as usize) / 2;
    let in_bss = (sbss as usize + ebss as usize) / 2;

    let cases = vec![in_rodata, in_text, in_data, in_bss];
    let writable = vec![false, false, true, true];
    let executable = vec![false, true, false, false];

    for (i, case) in cases.iter().enumerate() {
        let vpn = VirtAddr(*case).vpn();
        let pte = memory_set.translate(vpn).unwrap();
        assert!(pte.is_readalbe(), "should be readable");
        assert!(pte.is_writable() == writable[i], "writable failed");
        assert!(pte.is_executable() == executable[i], "executable failed");
        assert!(pte.page_number().0 == vpn.0, "direct map failed");
        debug!("kernel map test case-{} passed", i);
    }
    drop(memory_set);
    kernel!("kernel memory map test passed!");
}