use core::arch::asm;

pub unsafe fn write_medeleg(val: usize) {
    asm!("csrw medeleg, {}", in(reg) val)
}

pub unsafe fn write_mideleg(val: usize) {
    asm!("csrw mideleg, {}", in(reg) val)
}

//write thread pointer reg
pub unsafe fn write_tp(val: usize) {
    asm!("mv tp, {}", in(reg) val)
}

pub unsafe fn read_tp() -> usize {
    let tp: usize;
    asm!("mv {}, tp", out(reg) tp);
    return tp;
}