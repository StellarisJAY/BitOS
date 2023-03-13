use riscv::register::scause::{self, Trap::{Interrupt, Exception}};
use riscv::register::{stvec, sepc};
use riscv::register::scause::Interrupt::*;
use riscv::register::scause::Exception::*;
use context::TrapContext;

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
pub fn user_trap_handler() {
    let scause = scause::read();
    match scause.cause() {
        // 用户进程ecall导致的系统调用
        Exception(UserEnvCall) => {

        },
        // 由machine模式时间中断处理器抛出的S模式软件中断
        Interrupt(SupervisorSoft) => {

        },
        _ => panic!("unhandled trap"),
    }
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
    }
}