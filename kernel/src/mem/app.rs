use super::address::*;
use super::allocator::alloc;
use super::memory_set::*;
use super::page_table::*;
use crate::config::*;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT_BOTTOM};
use alloc::sync::Arc;
use alloc::vec;
use elf::endian::AnyEndian;
use elf::ElfBytes;

const PT_LOAD: u32 = 1;

// 应用程序虚拟地址空间布局：
//
// | .text | .data | heap | stacks... | trap_ctx ... | trampoline |
// heap大小固定，stack从高地址逆向增长
// stacks: 每个线程都有一个用户栈，位置 = base + tid * stack_size
// trap_ctx: 每个线程独有的陷入上下文，从高地址向低地址按tid逆向排列
// trampoline在虚拟页最高页，映射到.trampoline代码段
// kstask: 每个线程有一个内核栈，内核栈由内核的全局id分配，与tid无关
//
// memory set 只包括了进程相关的内存区域初始化，线程的栈和上下文映射在tcb创建时完成
impl MemorySet {
    // 从app的elf文件创建内存集合
    pub fn from_elf_data(data: &[u8]) -> (Self, usize, usize) {
        let mut memset = Self::new();
        let elf = ElfBytes::<AnyEndian>::minimal_parse(data);
        if let Err(error) = elf {
            error!("load elf error: {}", error);
            panic!("load elf data failed");
        }
        let elf = elf.unwrap();
        // 映射elf segments
        let segments = elf.segments().unwrap();
        let mut max_vpn = 0;
        for seg in segments {
            if seg.p_type == PT_LOAD {
                // vpn range
                let start_va = VirtAddr(seg.p_vaddr as usize);
                let end_va = VirtAddr(seg.p_vaddr as usize + seg.p_memsz as usize + PAGE_SIZE);
                let flags = elf_flags_to_pte_flags(seg.p_flags as usize);
                memset.insert_area(
                    MemoryArea::new(
                        start_va.vpn(),
                        end_va.vpn(),
                        MapMode::Indirect,
                        elf_flags_to_pte_flags(seg.p_flags as usize) | MemPermission::U.bits(),
                    ), // RWX flags
                    Some(elf.segment_data(&seg).unwrap()),
                ); // copy data
                max_vpn = end_va.vpn().0;
            }
        }
        memset.map_trampoline();
        // 用户空间的线程栈底：elf段之后的第一个页
        let stack_base = VirtPageNumber(max_vpn + 1).base_addr();
        return (memset, elf.ehdr.e_entry as usize, stack_base);
    }

    // 从父进程地址空间构建子进程地址空间
    // 由于引入了线程，只有调用fork的线程会被拷贝，所以这里不拷贝任何栈和trap上下文
    pub fn from_parent(parent: &MemorySet, stack_base: usize) -> Self {
        let mut memset = Self::new();
        parent.areas.iter().for_each(|area| {
            // 跳过ustack和trap_ctx
            let area_start = area.start_vpn.base_addr();
            if area_start < TRAMPOLINE && area_start >= stack_base {
                return;
            }
            let mut child_area =
                MemoryArea::new(area.start_vpn, area.end_vpn, area.mode, area.perm);
            // 将子进程的memset的vpn映射到父进程的物理页，并设置不可写（write触发PageFault，实现CopyOnWrite）
            for vpn in area.start_vpn.0..area.end_vpn.0 {
                let frame = area.frames.get(&VirtPageNumber(vpn)).unwrap();
                let flags = set_unwritable(area.perm);
                memset.page_table.map(VirtPageNumber(vpn), frame.ppn, flags);
                // 子进程要持有物理页的引用计数，避免父进程丢弃物理页后，物理页被自动回收
                child_area
                    .frames
                    .insert(VirtPageNumber(vpn), Arc::clone(&frame));
            }
            memset.areas.push(child_area);
        });

        // 子进程的栈与父进程相同，所以
        memset.map_trampoline();
        return memset;
    }

    // 删除可写权限，使写父进程和子进程写内存触发PageFault，然后在trap中进行CopyOnWrite
    pub fn remove_write_permission(&mut self) {
        self.areas.iter().for_each(|mut area| {
            // trap context 父子进程不共享
            if area.start_vpn != VirtAddr(TRAP_CONTEXT).vpn()
                && (MemPermission::W.bits() & area.perm != 0)
            {
                // 在页表上将每个vpn的pte设置不可写
                for vpn in area.start_vpn.0..area.end_vpn.0 {
                    let pte = self.page_table.translate(VirtPageNumber(vpn)).unwrap();
                    pte.set_unwritable();
                }
            }
        });
    }

    pub fn copy_on_write(&mut self, vpn: VirtPageNumber) -> bool {
        let mut vpn_valid = false;
        for area in self.areas.iter_mut() {
            if !area.frames.contains_key(&vpn) {
                continue;
            }
            vpn_valid = true;
            // 删除Arc<Frame>使物理页的引用计数减少，最终被回收
            let frame = area.frames.remove(&vpn).unwrap();
            // 新分配一个物理页，将原来物理页的数据拷贝
            let new_frame = alloc().unwrap();
            new_frame
                .ppn
                .as_bytes()
                .copy_from_slice(frame.ppn.as_bytes());
            // 修改页表，设置writable，设置新ppn
            let pte = self.page_table.translate(vpn).unwrap();
            pte.set_ppn(new_frame.ppn);
            pte.set_writable();
            area.frames.insert(vpn, Arc::new(new_frame));
            drop(frame);
            break;
        }
        return vpn_valid;
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

fn set_unwritable(flags: usize) -> usize {
    return flags & (!MemPermission::W.bits());
}

#[allow(unused)]
pub fn app_map_test(satp: usize) {
    debug!("testing app map");
    let page_table = PageTable::from_satp(satp);
    // 测试用例：.text, entry_point, trap_ctx
    let cases = vec![0x10200, 0x10208, TRAP_CONTEXT, TRAMPOLINE];
    let expect_exec = vec![true, true, false, true];
    let expect_read = vec![true, true, true, true];
    let expect_write = vec![false, false, true, false];
    let expect_umode = vec![true, true, false, false];

    for (i, case) in cases.iter().enumerate() {
        let pte = page_table.translate(VirtAddr(*case).vpn()).unwrap();
        assert!(pte.is_valid(), "pte should be valid");
        assert!(
            pte.is_usermode() == expect_umode[i],
            "pte shoule be usermode"
        );
        assert!(
            pte.is_executable() == expect_exec[i],
            "executable missmatch"
        );
        assert!(pte.is_readalbe() == expect_read[i], "readable missmatch");
        assert!(pte.is_writable() == expect_write[i], "writable missmatch");
        debug!("test case {} passed", i);
    }
}
