use super::address::*;
use super::memory_set::{MapMode, MemPermission, MemoryArea, MemorySet};
use super::page_table::PageTable;
use crate::arch::riscv::qemu::layout::UART0;
use crate::config::{KERNEL_STACK_BOTTOM, MAX_VA, PAGE_SIZE, PHYS_MEM_LIMIT, TRAMPOLINE};
use crate::driver::uart::put_char;
use alloc::vec;
use lazy_static::lazy_static;
use spin::mutex::SpinMutex;

// 内核内存集合
lazy_static! {
    pub static ref KERNEL_MEMSET: SpinMutex<MemorySet> = SpinMutex::new(MemorySet::new());
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn ekernel();
}

pub fn kernel_satp() -> usize {
    KERNEL_MEMSET.lock().satp()
}

#[no_mangle]
pub fn switch_to_kernel_space() {
    let satp = kernel_satp();
    unsafe {
        core::arch::asm!("csrw satp, {}", in(reg) satp);
        core::arch::asm!("sfence.vma");
    }
}

// 映射一个用户进程的内核栈
pub fn map_kernel_stack(bottom: usize, top: usize, data: Option<&[u8]>) {
    let mut memset = KERNEL_MEMSET.lock();
    memset.insert_area(
        MemoryArea::new(
            VirtAddr(bottom).vpn(),
            VirtAddr(top).vpn(),
            MapMode::Indirect, // 内核栈在高地址空间，已经超出物理地址范围，使用间接映射
            MemPermission::R.bits() | MemPermission::W.bits(),
        ),
        data,
    );
    drop(memset);
}

pub fn unmap_kernel_stack(bottom: usize) {
    let mut memset = KERNEL_MEMSET.lock();
    memset.remove_area(VirtAddr(bottom).vpn());
}

#[allow(unused)]
pub fn kernel_memset_translate(vpn: VirtPageNumber) -> Option<PhysPageNumber> {
    let memset = KERNEL_MEMSET.lock();
    let ppn = memset.vpn_to_ppn(vpn);
    drop(memset);
    return ppn;
}

impl MemorySet {
    // 初始化内核地址空间，将内核内存直接映射到页表
    pub fn init_kernel() {
        let mut memory_set = KERNEL_MEMSET.lock();
        memory_set.map_trampoline();
        // 映射低地址的mmio区域
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(UART0).vpn(),
                VirtAddr(stext as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            None,
        );
        // 内核.text段，R|X
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(stext as usize).vpn(),
                VirtAddr(etext as usize).vpn(),
                MapMode::Direct,
                MemPermission::X.bits() | MemPermission::R.bits(),
            ),
            None,
        );
        debug!(
            ".text section mapped, vpn range: [{}, {})",
            VirtAddr(stext as usize).vpn().0,
            VirtAddr(etext as usize).vpn().0
        );
        // 内核.rodata段，只读
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(srodata as usize).vpn(),
                VirtAddr(erodata as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits(),
            ),
            None,
        );
        debug!(
            ".rodata section mapped, vpn range: [{}, {})",
            VirtAddr(srodata as usize).vpn().0,
            VirtAddr(erodata as usize).vpn().0
        );
        // 内核.data段，可读可写
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(sdata as usize).vpn(),
                VirtAddr(edata as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            None,
        );
        debug!(
            ".data section mapped, vpn range: [{}, {})",
            VirtAddr(sdata as usize).vpn().0,
            VirtAddr(edata as usize).vpn().0
        );
        // 内核.bss段，可读可写
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(sbss as usize).vpn(),
                VirtAddr(ebss as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            None,
        );
        debug!(
            ".bss section mapped, vpn range: [{}, {})",
            VirtAddr(sbss as usize).vpn().0,
            VirtAddr(ebss as usize).vpn().0
        );
        // 映射内核段到栈之间的物理内存区域
        memory_set.insert_area(
            MemoryArea::new(
                VirtAddr(ekernel as usize).vpn(),
                VirtAddr(PHYS_MEM_LIMIT as usize).vpn(),
                MapMode::Direct,
                MemPermission::R.bits() | MemPermission::W.bits(),
            ),
            None,
        );
    }
}

#[allow(unused)]
pub fn kernel_map_test() {
    let satp = kernel_satp();
    let pagetable = PageTable::from_satp(satp);
    unsafe {
        let in_rodata = (srodata as usize + erodata as usize) / 2;
        let in_text = (stext as usize + etext as usize) / 2;
        let in_data = (sdata as usize + edata as usize) / 2;
        let in_bss = (sbss as usize + ebss as usize) / 2;
        let in_phys = ekernel as usize + 1024;
        let cases = vec![in_rodata, in_text, in_data, in_bss, in_phys];
        let writable = vec![false, false, true, true, true];
        let executable = vec![false, true, false, false, false];

        for (i, case) in cases.iter().enumerate() {
            let vpn = VirtAddr(*case).vpn();
            let pte = pagetable.translate(vpn).unwrap();
            assert!(pte.is_valid(), "should be valid");
            assert!(pte.is_readalbe(), "should be readable");
            assert!(pte.is_writable() == writable[i], "writable failed");
            assert!(pte.is_executable() == executable[i], "executable failed");
            assert!(pte.page_number().0 == vpn.0, "direct map failed");
            debug!("kernel map test case-{} passed", i);
        }
        kernel!("kernel memory map test passed!");
    }
}
