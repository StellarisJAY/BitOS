use super::address::*;
use super::allocator::{alloc, dealloc, Frame};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;

pub const SV39_PTE_PPN_BITS: usize = 44;
pub const SV39_PTE_FLAG_BITS: usize = 10;
const SV39_SATP_MODE: usize = 8;
const SV39_SATP_ASID_BITS: usize = 16;

//SV39 PTE，共64位，有效位54位
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

// 页表项 flags
bitflags! {
    pub struct PteFlags: usize {
        const V = 1<<0; // valid
        const R = 1<<1; // readable
        const W = 1<<2; // writable
        const X = 1<<3; // executable
        const U = 1<<4; // user mode accessible
        const G = 1<<5; // global pte
        const A = 1<<6; // recently accessed
        const D = 1<<7; // dirty
    }
}

#[repr(C)]
pub struct PageTable {
    pub root_ppn: PhysPageNumber,
    pub frames: Vec<Frame>, // frame保存页表使用的物理页的所有权
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNumber, flags: usize) -> Self {
        return Self {
            bits: (ppn.0 << SV39_PTE_FLAG_BITS) | (flags & ((1 << SV39_PTE_FLAG_BITS) - 1)),
        };
    }
    pub fn set_ppn(&mut self, ppn: PhysPageNumber) {
        self.bits = (ppn.0 << SV39_PTE_FLAG_BITS) | (self.bits & ((1 << SV39_PTE_FLAG_BITS) - 1));
    }

    pub fn set_flags(&mut self, flags: usize) {
        self.bits = self.bits | flags;
    }
    pub fn page_number(&self) -> PhysPageNumber {
        return PhysPageNumber((self.bits >> SV39_PTE_FLAG_BITS) & ((1 << SV39_PTE_PPN_BITS) - 1));
    }

    pub fn is_valid(&self) -> bool {
        return PteFlags::V.bits & self.bits != 0;
    }
    pub fn is_writable(&self) -> bool {
        return PteFlags::W.bits & self.bits != 0;
    }

    pub fn is_readalbe(&self) -> bool {
        return PteFlags::R.bits & self.bits != 0;
    }

    pub fn is_executable(&self) -> bool {
        return PteFlags::X.bits & self.bits != 0;
    }

    pub fn is_usermode(&self) -> bool {
        return PteFlags::U.bits & self.bits != 0;
    }

    pub fn set_unwritable(&mut self) {
        self.bits &= !PteFlags::W.bits;
    }

    pub fn set_writable(&mut self) {
        self.bits |= PteFlags::W.bits;
    }
}

impl PageTable {
    pub fn new() -> Self {
        let frame = alloc().unwrap();
        // 将frame所有权给pagetable，避免被回收
        return Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        };
    }

    pub fn from_satp(satp: usize) -> Self {
        return Self {
            root_ppn: PhysPageNumber(satp_ppn(satp)),
            frames: Vec::new(),
        };
    }

    // 将虚拟页映射到物理页
    pub fn map(&mut self, vpn: VirtPageNumber, ppn: PhysPageNumber, flags: usize) {
        let levels = divide_vpn(vpn);
        let mut current_ppn = self.root_ppn;
        let mut level = 0;
        for num in levels {
            let mut pte = &mut current_ppn.as_ptes()[num];
            // 遍历到最后一级页表，当前的pte的ppn指向vpn对应的物理页
            if level == levels.len() - 1 {
                // 该pte已经映射到某一个ppn，panic
                if pte.is_valid() {
                    error!("vpn {} already mapped", vpn.0);
                    panic!("vpn {} already mapped", vpn.0);
                } else {
                    // 将ppn写入叶子节点的pte
                    pte.set_ppn(ppn);
                    pte.set_flags(flags | PteFlags::V.bits());
                }
                break;
            }
            // 非叶子节点 pte无效，需要分配下一级页表的物理页
            if !pte.is_valid() {
                let frame = alloc().unwrap();
                pte.set_ppn(frame.ppn);
                pte.set_flags(PteFlags::V.bits());
                let old_ppn = current_ppn;
                current_ppn = frame.ppn;
                self.frames.push(frame);
            } else {
                current_ppn = pte.page_number();
            }
            level += 1;
        }
    }

    // 解除虚拟页在当前页表的映射
    pub fn unmap(&mut self, vpn: VirtPageNumber) {
        self.find_pte(vpn).map(|pte| {
            *pte = PageTableEntry::new(PhysPageNumber(0), 0);
        });
    }

    // 找到vpn对应的叶子节点页表项
    fn find_pte(&self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        let levels = divide_vpn(vpn);
        let mut ppn = self.root_ppn;
        let mut pte: &mut PageTableEntry;
        let mut i = 0;
        for num in levels {
            pte = &mut ppn.as_ptes()[num];
            if i == levels.len() - 1 {
                return Some(pte);
            }
            if !pte.is_valid() {
                debug!("find pte break at level: {}, pte bits: {:#b}", i, pte.bits);
                break;
            } else {
                ppn = pte.page_number();
            }
            i = i + 1;
        }
        return None;
    }

    // vpn 转换 ppn
    pub fn vpn_to_ppn(&self, vpn: VirtPageNumber) -> Option<PhysPageNumber> {
        return self.find_pte(vpn).map(|pte| {
            return pte.page_number();
        });
    }

    // 虚拟地址转换物理地址
    pub fn va_to_pa(&self, va: VirtAddr) -> Option<PhysAddr> {
        return self
            .find_pte(va.vpn())
            .map(|pte| PhysAddr(pte.page_number().base_addr() + va.offset()));
    }

    pub fn translate(&self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        return self.find_pte(vpn);
    }

    // 获取SV39页表的satp值
    pub fn satp(&self, asid: usize) -> usize {
        return self.root_ppn.0
            | (asid << SV39_PTE_PPN_BITS)
            | (SV39_SATP_MODE << (SV39_PTE_PPN_BITS + SV39_SATP_ASID_BITS));
    }
}

fn satp_ppn(satp: usize) -> usize {
    return satp & ((1 << SV39_PTE_PPN_BITS) - 1);
}

// 将SV39的27位vpn分割成3个9bits的多级vpn
fn divide_vpn(vpn: VirtPageNumber) -> [usize; 3] {
    let mut number = vpn.0;
    let mut parts = [0usize; 3];
    let mut i = 2;
    while number > 0 {
        parts[i] = number & 0x1ff;
        number = number >> 9;
        i = i - 1;
    }
    return parts;
}
