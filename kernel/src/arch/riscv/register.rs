use super::qemu::layout::{CLINT0, CLINT_MTIME, CLINT_MTIMECMP};
use core::arch::asm;
use core::ptr;
#[inline]
pub unsafe fn write_medeleg(val: usize) {
    asm!("csrw medeleg, {}", in(reg) val)
}

#[inline]
pub unsafe fn write_mideleg(val: usize) {
    asm!("csrw mideleg, {}", in(reg) val)
}

//write thread pointer reg
#[inline]
pub unsafe fn write_tp(val: usize) {
    asm!("mv tp, {}", in(reg) val)
}
#[inline]
pub unsafe fn read_tp() -> usize {
    let tp: usize;
    asm!("mv {}, tp", out(reg) tp);
    return tp;
}

#[inline]
pub unsafe fn read_mtime() -> usize {
    ptr::read_volatile(CLINT_MTIME as *const usize)
}

pub unsafe fn read_mtimecmp(mhartid: usize) -> usize {
    ptr::read_volatile((CLINT_MTIMECMP + 8 * mhartid) as *const usize)
}

unsafe fn write_mtimecmp(mhartid: usize, value: usize) {
    let offset = CLINT_MTIME + 8 * mhartid;
    ptr::write_volatile(offset as *mut usize, value);
}

pub unsafe fn add_mtimecmp(mhartid: usize, interval: usize) {
    let value = read_mtime();
    // 下一个中断的时间：当前time+间隔
    write_mtimecmp(mhartid, value + interval);
}

pub fn mtime_cmp_addr(mhartid: usize) -> usize {
    return CLINT_MTIMECMP + 8 * mhartid;
}
