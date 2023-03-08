use crate::register;

pub mod context;
pub mod pcb;

pub fn cpuid() -> usize {
    unsafe {
        register::read_tp()
    }
}