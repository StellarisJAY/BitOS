use super::address::*;
use super::allocator::{alloc, dealloc, Frame};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::page_table::{PageTable, PageTableEntry};
use bitflags::bitflags;
use crate::config::{PAGE_SIZE, TRAMPOLINE};

bitflags! {
    pub struct MemPermission: usize {
        const R = 1<<1;
        const W = 1<<2;
        const X = 1<<3;
    }
}

// 虚拟内存到物理内存的映射方式
#[derive(PartialEq, Eq)]
pub enum MapMode {
    Direct,    // 直接映射：vpn = ppn
    Indirect,  // 间接映射：vpn != ppn, rand(ppn)
}

// MemoryArea 内存段，一个连续的虚拟内存区域
pub struct MemoryArea {
    pub start_vpn: VirtPageNumber,
    pub end_vpn: VirtPageNumber,
    pub frames: BTreeMap<VirtPageNumber, Frame>,  // frames集合，保存内存段拥有的所有物理页
    pub mode: MapMode,                            // 内存段映射模式
    pub perm: usize,
}

// MemorySet 内存集合，多个内存段的集合，通过相同的页表映射
pub struct MemorySet {
    pub page_table: PageTable,
    pub areas: Vec<MemoryArea>,
}

impl MemoryArea {
    pub fn new(start_vpn: VirtPageNumber, end_vpn: VirtPageNumber, mode: MapMode, perm: usize) -> Self {
        return Self { start_vpn: start_vpn, end_vpn: end_vpn, frames: BTreeMap::new(), mode: mode, perm: perm };
    }
    // 将当前内存段的vpn范围映射到指定的页表，必要时拷贝数据
    pub fn map(&mut self, page_table: &mut PageTable, data: Option<&[u8]>) {
        let mut offset = 0;
        for vpn in self.start_vpn.0..self.end_vpn.0 {
            if self.mode == MapMode::Direct {
                page_table.map(VirtPageNumber(vpn), PhysPageNumber(vpn), self.perm);
            }else {
                // 非直接映射，需要新的物理页
                let frame = alloc().unwrap();
                page_table.map(VirtPageNumber(vpn), frame.ppn, self.perm);
                // 拷贝数据到当前的物理页
                if let Some(bytes) = data {
                    let limit = (offset + PAGE_SIZE).min(bytes.len());
                    frame.ppn.as_bytes()[0..(limit - offset)].copy_from_slice(&bytes[offset..limit]);
                    offset = limit;
                }
                self.frames.insert(VirtPageNumber(vpn), frame);
            }
        }
    }
    // 解除内存区域在该页表上的映射
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.start_vpn.0..self.end_vpn.0 {
            page_table.unmap(VirtPageNumber(vpn));
            self.frames.remove(&VirtPageNumber(vpn));
        }
    }
}

impl MemorySet {
    pub fn new() -> Self {
        return Self {page_table: PageTable::new(), areas: Vec::new()};
    }

    // 内存集合中插入一个内存段
    pub fn insert_area(&mut self, mut area: MemoryArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table, data);
        self.areas.push(area);
    }
    
    pub fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        // 所在地址空间虚拟地址最高页，映射到trampoline代码的物理地址
        self.page_table.map(
                VirtAddr(TRAMPOLINE).vpn(),
                PhysAddr(strampoline as usize).page_number(),
                MemPermission::R.bits | MemPermission::X.bits);
    }

    pub fn translate(&self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        return self.page_table.translate(vpn);
    }
    
    pub fn vpn_to_ppn(&self, vpn: VirtPageNumber) -> Option<PhysPageNumber> {
        return self.page_table.vpn_to_ppn(vpn);
    }

    pub fn va_to_pa(&self, va: VirtAddr) -> Option<PhysAddr> {
        return self.page_table.va_to_pa(va);
    }

    pub fn satp(&self) -> usize {
        return self.page_table.satp(0);
    }

    pub fn reset_satp(&self) {
        let satp = self.satp();
        unsafe {
            core::arch::asm!("csrw satp, {}", in(reg) satp);
            core::arch::asm!("sfence.vma zero, zero");
        }
    }

    // 虚拟地址的连续buffer，转换成物理页的切片集合
    pub fn translate_buffer(&self, addr: usize, len: usize) -> Vec<&'static [u8]> {
        let (start_va, end_va) = (VirtAddr(addr), VirtAddr(addr + len));
        let mut vpn = start_va.vpn().0;
        let end_vpn = end_va.vpn().0;
        let mut start_off = start_va.offset();
        let mut end_off = PAGE_SIZE;
        let mut buffers: Vec<&'static [u8]> = Vec::new();
        while vpn <= end_vpn {
            if vpn == end_vpn {
                end_off = end_va.offset();
            }
            let ppn = self.vpn_to_ppn(VirtPageNumber(vpn)).unwrap();
            buffers.push(&ppn.as_bytes()[start_off..end_off]);
            start_off = 0;
        }
        return buffers;
    }
}

// 在debug监控MemoryArea的所有权丢失
impl Drop for MemoryArea {
    fn drop(&mut self) {
        debug!("memory area {}, {} dropped", self.start_vpn.0, self.end_vpn.0);
    }
}
