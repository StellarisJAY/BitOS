#![no_std]
#![no_main]

use core::arch::global_asm;
use core::arch::asm;
use riscv::register::*;
use proc::cpuid;

mod config;
mod register;
mod proc;

global_asm!(include_str!("asm/entry.S"));

#[no_mangle]
pub fn rust_start() {
    unsafe {
        // 设置mstatus， 使mret 返回supervisor模式
        mstatus::set_mpp(mstatus::MPP::Supervisor);
        // 设置mepc，mret的跳转到rust_main
        mepc::write(rust_main as usize);
        satp::write(0);
        // medeleg and mideleg
        asm!("csrw medeleg, {}", in(reg) 0xffff);
        asm!("csrw mideleg, {}", in(reg) 0xffff);
        // supervisor处理异常：soft、external、timer
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
        // 设置物理地址范围
        pmpaddr0::write(config::PHYS_MEM_LIMIT - 1);
        // 物理地址保护，RWX=1, A=TOR, 范围[0,pmpaddr0)
        pmpcfg0::write(0b1111);

        // 将hartid保存到tp寄存器（rust_main中会使用）
        let cpuid = mhartid::read();
        asm!("mv tp, {}", in(reg) cpuid);
        asm!("mret");
    }
}

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
static KERNEL_INITED: AtomicU8 = AtomicU8::new(0);

#[no_mangle]
pub fn rust_main() {
    if cpuid() == 0 {
        // cpu0 init kernel
        KERNEL_INITED.store(1, Ordering::SeqCst);
    }else {
        // other cpu wait cpu0
        while KERNEL_INITED.load(Ordering::SeqCst) == 0 {}
    }
    // schedule proc
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> !{
    panic!("unavailable");
}

