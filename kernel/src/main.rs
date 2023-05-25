#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(drain_filter)]
#![allow(unused)]
use arch::riscv::register::*;
use config::CPUS;
use core::arch::asm;
use core::arch::global_asm;
use riscv::register::*;
use task::scheduler::cpuid;
extern crate alloc;

#[macro_use]
mod console;
mod arch;
mod config;
mod driver;
mod fs;
mod mem;
mod proc;
mod sync;
mod syscall;
mod task;
mod trap;

global_asm!(include_str!("asm/entry.S"));
global_asm!(include_str!("asm/kernelvec.S"));
global_asm!(include_str!("asm/trampoline.S"));
global_asm!(include_str!("asm/switch.S"));
global_asm!(include_str!("asm/link_kernel_app.S"));

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
        timer_init();
        // 将hartid保存到tp寄存器
        let cpuid = mhartid::read();
        asm!("mv tp, {}", in(reg) cpuid);
        asm!("mret");
    }
}

static mut TIMER_SCRATCH: [[usize; 5]; CPUS] = [[0usize; 5]; CPUS];

// 初始化m时钟中断
unsafe fn timer_init() {
    // 每个cpu单独处理时间中断
    let id = mhartid::read();
    // 向clint提交中断间隔
    let interval = 1000000;
    add_mtimecmp(id, interval);

    // scratch[0..=2] 用于保存寄存器
    // scratch[3] : CLINT MTIMECMP地址
    // scratch[4] : 时钟中断间隔
    TIMER_SCRATCH[id][3] = mtime_cmp_addr(id);
    TIMER_SCRATCH[id][4] = interval;

    // mscratch寄存器记录scratch数组地址
    mscratch::write(TIMER_SCRATCH[id].as_ptr() as usize);

    // 设置机器中断处理器为timer_vec
    extern "C" {
        fn timervec();
    }
    mtvec::write(timervec as usize, mtvec::TrapMode::Direct);
    // 开启machine中断
    mstatus::set_mie();
    // 开启machine模式时钟中断
    mie::set_mtimer();
}

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
static KERNEL_INITED: AtomicU8 = AtomicU8::new(0);

#[no_mangle]
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
        proc::init_processors();
        mem::kernel::switch_to_kernel_space();
        kernel!("hart0 booted, kernel initialized");
        fs::inode::list_apps();
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

    loop {}
}
