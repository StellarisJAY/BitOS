use crate::register;

pub mod context;
pub mod pcb;
pub mod pid;
pub mod scheduler;
pub mod loader;

pub fn cpuid() -> usize {
    unsafe {
        register::read_tp()
    }
}