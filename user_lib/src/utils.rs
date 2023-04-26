use core::fmt::*;
use crate::syscall::write;
use crate::syscall::read;
const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

struct Stdout {}

static mut STDOUT: Stdout = Stdout{};

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        write(FD_STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: Arguments) {
    unsafe {
        STDOUT.write_fmt(args).unwrap();
    }
}

pub fn put_char(c: u8) {
    let buf = [c; 1];
    let _ = write(FD_STDOUT, &buf);
}

pub fn get_char() -> u8 {
    let buf = [0u8; 1];
    let _ = read(FD_STDIN, &buf);
    return buf[0];
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::utils::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::utils::print(format_args!(concat!($fmt) $(, $($arg)+)?));
    }
}

