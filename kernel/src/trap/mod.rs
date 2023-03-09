use riscv::register::scause::{self, Trap::{Interrupt, Exception}};
use riscv::register::stvec;

pub mod context;

pub fn trap_init() {
    unsafe {
        // stvec寄存器设置为trap_handler
        stvec::write(trap_handler as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler() {
    let cause = scause::read();
    // 临时trap_handler
    match cause.cause() {
        Interrupt(e) => {
            println!("interrupt, code: {}", cause.code());
        },
        Exception(e) => {
            panic!("exception, code: {}", cause.code());
        },
    }
}