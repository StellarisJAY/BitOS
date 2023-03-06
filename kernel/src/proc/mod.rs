use crate::register;

pub fn cpuid() -> usize {
    unsafe {
        register::read_tp()
    }
}