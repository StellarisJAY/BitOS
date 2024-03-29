use crate::arch::riscv::register::clear_sip_soft;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::driver::plic::handle_irq;
use crate::mem::address::VirtAddr;
use crate::syscall::handle_syscall;
use crate::task::scheduler::{
    current_task, current_task_satp, current_task_trap_context, current_task_trap_va,
    yield_current_task,
};
use context::TrapContext;
use core::arch::asm;
use riscv::register::scause::Exception::*;
use riscv::register::scause::Interrupt::*;
use riscv::register::scause::{
    self,
    Trap::{Exception, Interrupt},
};
use riscv::register::{sepc, sstatus, stval, stvec};
pub mod context;

extern "C" {
    fn _user_vec();
    fn _kernel_vec();
}

pub fn trap_init() {
    unsafe {
        // stvec寄存器设置为trap_handler
        stvec::write(_kernel_vec as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub unsafe fn user_trap_handler() {
    let mut ctx = current_task_trap_context();
    let scause = scause::read();
    let val = stval::read();
    match scause.cause() {
        // 用户进程ecall导致的系统调用
        Exception(UserEnvCall) => {
            // epc + 4使trap结束后能够跳到trap之后的一条指令
            ctx.sepc += 4;
            let ret = handle_syscall(ctx.a[7], [ctx.a[0], ctx.a[1], ctx.a[2]]);
            // 因为syscall可能切换了另一个进程，所以这里要重新获取ctx
            ctx = current_task_trap_context();
            ctx.a[0] = ret as usize;
        }
        Exception(IllegalInstruction) => {
            kernel!("user illegal instruction, stval: {}", val);
            panic!("illegal instruction")
        }
        Exception(LoadPageFault) => {
            kernel!("user load page fault, stval: {:#x}", val);
            panic!("load page fault")
        }
        Exception(LoadFault | StoreFault) => {
            kernel!("user load/store fault, stval: {:#x}", val);
            panic!("load/store fault")
        }
        Interrupt(SupervisorSoft) => {
            // 清除sip的soft中断，避免重复中断
            clear_sip_soft();
            yield_current_task();
        }
        Exception(StorePageFault) => {
            let pcb = current_task().inner.borrow().process.upgrade().unwrap();
            if !pcb.copy_on_write(VirtAddr(val).vpn()) {
                kernel!("store page fault, va: {:#x}", val);
                panic!("store page fault");
            }
        }
        // 外设中断
        Interrupt(SupervisorExternal) => {
            handle_irq();
        }
        _ => {
            kernel!("{:?}, stval: {:#x}", scause.cause(), val);
            panic!("unhandled trap")
        }
    }
    user_trap_return();
}

#[no_mangle]
pub fn kernel_trap_handler() {
    let scause = scause::read();
    let val = stval::read();
    match scause.cause() {
        Exception(e) => {
            kernel!("exception: {:?}, val: {:#x}", e, val);
            panic!("kernel exception")
        },
        _ => panic!("unhandled trap"),
    }
}

#[no_mangle]
pub fn user_trap_return() {
    extern "C" {
        fn _user_ret(ctx: *const TrapContext, satp: usize);
        fn _user_vec();
    }
    let user_ret_va = _user_ret as usize - _user_vec as usize + TRAMPOLINE;
    let satp = current_task_satp();
    let trap_context = current_task_trap_va();
    unsafe {
        // 设置User模式trap处理器
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
        // 切换回User模式
        sstatus::set_spp(sstatus::SPP::User);
        // 使用jr指令直接跳转到地址
        asm!("jr {user_ret_va}",
            user_ret_va = in(reg) user_ret_va,
            in("a0") trap_context, // a0 为TRAP_CONTEXT的固定虚拟地址
            in("a1") satp,   // a1 寄存器写入用户地址空间的satp，即用户地址空间的页表ppn
            options(noreturn)
        );
    }
}
