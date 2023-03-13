use core::arch::asm;
use super::qemu::layout::{CLINT0,  CLINT_MTIME, CLINT_MTIMECMP};
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
unsafe fn read_mtime() -> usize {
    ptr::read_volatile(CLINT_MTIME as *const usize)
}

unsafe fn write_mtimecmp(mhartid:usize, value: usize) {
    let offset = CLINT_MTIME + 8*mhartid;
    ptr::write_volatile(offset as *mut usize, value);
}

pub unsafe fn add_mtimecmp(mhartid:usize, interval:usize){
    let value = read_mtime();
    write_mtimecmp(mhartid, value+interval);
}

pub fn mtie_cmp_addr(mhartid:usize) -> usize{
    return CLINT0 + 8*mhartid + 0x4000;
}