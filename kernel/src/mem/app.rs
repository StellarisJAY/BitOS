use super::address::*;
use super::memory_set::*;
use elf::ElfBytes;
use elf::endian::AnyEndian;
use crate::config::*;

const PT_LOAD: u32 = 0;

impl MemorySet {
    // 从app的elf文件创建内存集合
    pub fn from_elf_data(data: &[u8]) -> Self {
        let mut memset = Self::new();
        let elf = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();
        // 映射elf segments
        let segments = elf.segments().unwrap();
        let mut max_vpn = 0;
        for seg in segments {
            if seg.p_type == PT_LOAD {
                // vpn range
                let start_va = VirtAddr(seg.p_vaddr as usize);
                let end_va = VirtAddr(seg.p_vaddr as usize + seg.p_filesz as usize);
                memset.insert_area(MemoryArea::new(
                        start_va.vpn(),
                        end_va.vpn(),
                        MapMode::Indirect,
                        (seg.p_flags & 0b111) as usize), // RWX flags
                Some(elf.segment_data(&seg).unwrap())); // copy data
                max_vpn = end_va.vpn().0;
            }
        }
        // 映射用户空间栈，起始地址与elf结束页中间间隔一个空页（可以通过缺页中断捕捉内存访问越界）
        let stack_bottom_vpn = VirtPageNumber(max_vpn + 2);
        let stack_top_vpn = VirtAddr(stack_bottom_vpn.base_addr() + USER_STACK_SIZE - 1).vpn();
        memset.insert_area(MemoryArea::new(
                stack_bottom_vpn,
                stack_top_vpn,
                MapMode::Indirect,
                MemPermission::R.bits() | MemPermission::W.bits()
        ), None);
        // 映射Trampoline
        memset.map_trampoline();
        return memset;
    }
}