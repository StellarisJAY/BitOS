use crate::driver::uart::UART;
use core::fmt::*;

pub fn print(args: Arguments) {
    let mut uart = UART.lock();
    uart.write_fmt(args).unwrap();
    drop(uart);
}

pub fn print_str(s: &str) {
    let mut uart = UART.lock();
    uart.write_str(s);
    drop(uart);
}

pub fn print_buf(buf: &[u8]) {
    let mut uart = UART.lock();
    for b in buf.iter() {
        uart.put(*b);
    }
    drop(uart);
}

pub fn get_char() -> Option<u8> {
    let mut uart = UART.lock();
    uart.get()
}

pub fn debug(args: Arguments) {
    if crate::config::DEBUG_MODE {
        print_str("[debug] ");
        print(args);
        print_str("\n");
    }
}

pub fn print_banner() {
    let banner = "______ _________________________
___  /____(_)_  /__  __ \\_  ___/
__  __ \\_  /_  __/  / / /____ \\
_  /_/ /  / / /_ / /_/ /____/ /
/_.___//_/  \\__/ \\____/ /____/ \n";
    print_str(banner);
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print_str("[info]  ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\n");
    }
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print_str("[error] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\n");
    }
}

#[macro_export]
macro_rules! kernel {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print_str("[kernel] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\n");
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::debug(format_args!($fmt $(, $($arg)+)?));
    }
}
