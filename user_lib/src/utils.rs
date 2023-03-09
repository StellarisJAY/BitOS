use core::fmt::*;
use crate::syscall::write;
const FD_STDOUT: usize = 1;

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

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::utils::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

