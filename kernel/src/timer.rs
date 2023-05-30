use crate::config::{CPUS, TIME_FREQ};
use crate::arch::riscv::register::*;
use riscv::register::{mhartid, mscratch, mtvec, mstatus, mie};

static mut TIMER_SCRATCH: [[usize; 5]; CPUS] = [[0; 5]; CPUS];

pub unsafe fn timer_init() {
    let id = mhartid::read();
    let interval = TIME_FREQ / 10;
    add_timer(id, interval);
    TIMER_SCRATCH[id][3] = mtime_cmp_addr(id);
    TIMER_SCRATCH[id][4] = interval;
    mscratch::write(TIMER_SCRATCH[id].as_ptr() as usize);
    
    extern "C" {
        fn timervec();
    }
    
    mtvec::write(timervec as usize, riscv::register::utvec::TrapMode::Direct);
    
    mstatus::set_mie();
    mie::set_mtimer();
}

pub fn get_time() -> usize {
    unsafe {read_mtime()}
}

pub fn add_timer(hartid: usize, interval: usize) {
    unsafe {
        add_mtimecmp(hartid, interval);
    }
}

pub fn get_next_trigger(mhartid: usize) -> usize {
    unsafe {
        read_mtimecmp(mhartid)
    }
}
