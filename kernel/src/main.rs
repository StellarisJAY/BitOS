#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(drain_filter)]
#![allow(unused)]
use core::arch::global_asm;
use task::scheduler::cpuid;
extern crate alloc;

#[macro_use]
mod console;
mod arch;
mod boot;
mod config;
mod driver;
mod fs;
mod ipc;
mod mem;
mod proc;
mod shutdown;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("asm/entry.S"));
global_asm!(include_str!("asm/kernelvec.S"));
global_asm!(include_str!("asm/trampoline.S"));
global_asm!(include_str!("asm/switch.S"));
global_asm!(include_str!("asm/link_fs.S"));

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;

static KERNEL_INITED: AtomicU8 = AtomicU8::new(0);

pub unsafe fn rust_main() {
    let id = cpuid();
    if id == 0 {
        // cpu0 init kernel
        driver::uart::Uart::init();
        console::print_banner();
        trap::trap_init();
        mem::init();
        kernel!("kernel memory initialized");
        driver::init();
        kernel!("drivers initialized");
        proc::init_proc();
        mem::kernel::switch_to_kernel_space();
        kernel!("hart0 booted, kernel initialized");
        KERNEL_INITED.store(1, Ordering::SeqCst);
    } else {
        while KERNEL_INITED.load(Ordering::SeqCst) == 0 {}
    }
    task::scheduler::schedule();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    match info.location() {
        Some(loc) => {
            if let Some(msg) = info.message() {
                error!(
                    "kernel panicked at {}:{}, message: {}",
                    loc.file(),
                    loc.line(),
                    msg.as_str().unwrap_or("no message")
                );
            } else {
                error!("kernel panicked at {}:{}", loc.file(), loc.line());
            }
        }
        None => {
            error!("kernel panicked");
        }
    }
    shutdown::panic_shutdown();
    loop {}
}
