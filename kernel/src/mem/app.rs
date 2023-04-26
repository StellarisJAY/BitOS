use super::address::*;
use super::memory_set::*;
use elf::ElfBytes;
use elf::endian::AnyEndian;
use crate::config::*;
use super::page_table::*;
use alloc::vec;
const PT_LOAD: u32 = 1;

// 应用程序虚拟地址空间布局：
//
// | .text | .data | heap ... stack | trap_ctx | trampoline |
// heap大小固定，stack从高地址逆向增长
// trap虚拟地址固定，U切换S时保存寄存器
// trampoline在虚拟页最高页，映射到.trampoline代码段
impl MemorySet {
    // 从app的elf文件创建内存集合
    pub fn from_elf_data(data: &[u8]) -> (Self, usize, usize) {
        let mut memset = Self::new();
        let elf = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();
        // 映射elf segments
        let segments = elf.segments().unwrap();
        let mut max_vpn = 0;
        for seg in segments {
            if seg.p_type == PT_LOAD {
                // vpn range
                let start_va = VirtAddr(seg.p_vaddr as usize);
                let end_va = VirtAddr(seg.p_vaddr as usize + seg.p_memsz as usize + PAGE_SIZE);
                let flags = elf_flags_to_pte_flags(seg.p_flags as usize);
                memset.insert_area(MemoryArea::new(
                    start_va.vpn(),
                        end_va.vpn(),
                        MapMode::Indirect,
                        elf_flags_to_pte_flags(seg.p_flags as usize) | MemPermission::U.bits()), // RWX flags
                Some(elf.segment_data(&seg).unwrap())); // copy data
                max_vpn = end_va.vpn().0;
            }
        }
        // 在高地址映射用户栈
        let (stack_bottom, stack_top) = user_stack_position();
        memset.insert_area(MemoryArea::new(
                VirtAddr(stack_bottom).vpn(),
                VirtAddr(stack_top).vpn(),
                MapMode::Indirect,
                MemPermission::R.bits() | MemPermission::W.bits() | MemPermission::U.bits() // 用户栈设置U mode，只允许用户模式访问
        ), None);
        // 映射Trampoline
        memset.map_trampoline();
        // 映射TrapContext
        memset.insert_area(MemoryArea::new(
            VirtAddr(TRAP_CONTEXT).vpn(),
                VirtAddr(TRAP_CONTEXT + PAGE_SIZE).vpn(),
                MapMode::Indirect,
                MemPermission::R.bits() | MemPermission::W.bits()), None); // trap_ctx 只在Supervisor访问，不设置U mode
        return (memset, elf.ehdr.e_entry as usize, stack_top);
    }
}

// elf flags 转换 pte flags
fn elf_flags_to_pte_flags(p_flags: usize) -> usize {
    // elf中的段全部是User mode访问
    let mut flags: usize = MemPermission::U.bits();
    if p_flags & 1 != 0 {
        flags |= MemPermission::X.bits();
    }
    if p_flags & 2 != 0 {
        flags |= MemPermission::W.bits();
    }
    if p_flags & 4 != 0 {
        flags |= MemPermission::R.bits();
    }
    return flags;
}

#[allow(unused)]
pub fn app_map_test(satp: usize) {
    debug!("testing app map");
    let page_table = PageTable::from_satp(satp);
    let (stack_bottom, _) = user_stack_position();
    // 测试用例：.text, entry_point, trap_ctx, user_stack
    let cases = vec![0x10200, 0x10208, TRAP_CONTEXT, stack_bottom + PAGE_SIZE, TRAMPOLINE];
    let expect_exec = vec![true, true, false, false, true];
    let expect_read = vec![true, true, true, true, true];
    let expect_write = vec![false, false, true, true, false];
    let expect_umode = vec![true, true, false, true, false];

    for (i, case) in cases.iter().enumerate() {
        let pte = page_table.translate(VirtAddr(*case).vpn()).unwrap();
        assert!(pte.is_valid(), "pte should be valid");
        assert!(pte.is_usermode() == expect_umode[i], "pte shoule be usermode");
        assert!(pte.is_executable() == expect_exec[i], "executable missmatch");
        assert!(pte.is_readalbe() == expect_read[i], "readable missmatch");
        assert!(pte.is_writable() == expect_write[i], "writable missmatch");
        debug!("test case {} passed", i);
    }
}