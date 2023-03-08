use super::address::*;
use super::allocator::{alloc, dealloc, Frame};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::page_table::{PageTable};
use bitflags::bitflags;

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
    // 将当前内存段的vpn范围映射到指定的页表
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.start_vpn.0..self.end_vpn.0 {
            if self.mode == MapMode::Direct {
                page_table.map(VirtPageNumber(vpn), PhysPageNumber(vpn), self.perm);
            }else {
                // 非直接映射，需要新的物理页
                let frame = alloc().unwrap();
                page_table.map(VirtPageNumber(vpn), frame.ppn, self.perm);
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