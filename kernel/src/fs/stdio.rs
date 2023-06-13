use super::{File, UserBuffer};
use alloc::vec::Vec;
use crate::console::{print_buf, get_char};
use crate::task::scheduler::yield_current_task;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn read<'a>(&self, mut buf: &mut UserBuffer) -> usize {
        assert!(buf.length() == 1, "only support 1 characters per read");
        loop {
            if let Some(ch) = get_char() {
                let data: [u8; 1] = [ch];
                buf.write(0, &data);
                break;
            }else {
                yield_current_task();
                continue;
            }
        }
        0
    }
    
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize {
        panic!("can not write stdin")
    }
}

impl File for Stdout {
    fn read<'a>(&self, buf: &mut UserBuffer) -> usize {
        panic!("can not read stdout")
    }
    
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize {
        let mut sum = 0;
        buf.foreach(|bytes| {
            print_buf(bytes);
            sum += bytes.len();
        });
        return sum;
    }
}