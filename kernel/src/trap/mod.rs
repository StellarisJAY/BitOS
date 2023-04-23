use riscv::register::scause::{self, Trap::{Interrupt, Exception}};
use riscv::register::{stvec, sepc, sstatus};
use riscv::register::scause::Interrupt::*;
use riscv::register::scause::Exception::*;
use context::TrapContext;
use crate::proc::scheduler::{current_proc_trap_context, current_proc_satp, current_proc_trap_addr};
use crate::syscall::handle_syscall;
use crate::config::TRAMPOLINE;
use core::arch::asm;

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
    let mut ctx = current_proc_trap_context();
    let scause = scause::read();
    match scause.cause() {
        // 用户进程ecall导致的系统调用
        Exception(UserEnvCall) => {
            // epc + 4使trap结束后能够跳到trap之后的一条指令
            ctx.sepc += 4;
            let ret = handle_syscall(ctx.a[7], [ctx.a[0], ctx.a[1], ctx.a[2]]);
            // 因为syscall可能切换了另一个进程，所以这里要重新获取ctx
            ctx = current_proc_trap_context();
            ctx.a[0] = ret as usize;
        },
        // 由machine模式时间中断处理器抛出的S模式软件中断
        Interrupt(SupervisorSoft) => {

        },
        _ => panic!("unhandled trap"),
    }
    user_trap_return();
}

#[no_mangle]
pub fn kernel_trap_handler() {
    let scause = scause::read();
    let mut epc = sepc::read();
    epc += 4;
    sepc::write(epc);
    match scause.cause() {
        // 由machine模式时间中断处理器抛出的S模式软件中断
        Interrupt(e) => {
            debug!("interrupt: {:?}", e);
        },
        Exception(e) => {
            debug!("exception: {:?}", e);
        }
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
    let satp = current_proc_satp();
    let trap_context = current_proc_trap_addr() as *const TrapContext;
    unsafe {
        // 设置User模式trap处理器
        stvec::write(_user_vec as usize, stvec::TrapMode::Direct);
        // 切换回User模式
        sstatus::set_spp(sstatus::SPP::User);
        _user_ret(trap_context, satp);
    }
}