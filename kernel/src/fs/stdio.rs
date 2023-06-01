use super::File;
use alloc::vec::Vec;
use crate::console::{print_buf, get_char};
use crate::task::scheduler::yield_current_task;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn read<'a>(&self, mut buf: Vec<&'a mut [u8]>) -> usize {
        assert!(buf.len() == 1, "only support 1 characters per read");
        loop {
            if let Some(ch) = get_char() {
                unsafe {buf[0].as_mut_ptr().write_volatile(ch);}
                break;
            }else {
                yield_current_task();
                continue;
            }
        }
        0
    }
    
    fn write<'a>(&self, buf: Vec<&'a mut [u8]>) -> usize {
        panic!("can not write stdin")
    }
}

impl File for Stdout {
    fn read<'a>(&self, buf: Vec<&'a mut [u8]>) -> usize {
        panic!("can not read stdout")
    }
    
    fn write<'a>(&self, buf: Vec<&'a mut [u8]>) -> usize {
        let mut sum = 0;
        for bytes in buf {
            print_buf(bytes);
            sum += bytes.len();
        }
        return sum;
    }
}