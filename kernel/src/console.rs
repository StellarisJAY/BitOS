use core::fmt::*;
use crate::driver::uart::{UART};

pub fn print(args: Arguments) {
    let mut uart = UART.lock();
    uart.write_fmt(args).unwrap();
    drop(uart);
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
