#![no_std]
#![no_main]

#![allow(unused)]
use core::arch::global_asm;
use core::arch::asm;
use riscv::register::*;
use proc::cpuid;

extern crate alloc;

mod config;
mod register;
mod driver;
mod proc;
#[macro_use]
mod console;
mod trap;
mod mem;

global_asm!(include_str!("asm/entry.S"));

// 引导内核启动，设置M模式下的寄存器，之后跳转到内核入口进入S模式
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

        // 将hartid保存到tp寄存器
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
    let id = cpuid();
    if id == 0 {
        // cpu0 init kernel
        driver::init();
        console::print_banner();
        kernel!("drivers initialized");
        kernel!("hart0 booted, kernel initialized");
        KERNEL_INITED.store(1, Ordering::SeqCst);
    }else {
        while KERNEL_INITED.load(Ordering::SeqCst) == 0 {}
    }
    // schedule proc
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> !{
    error!("kernel paniced");
    panic!()
}

