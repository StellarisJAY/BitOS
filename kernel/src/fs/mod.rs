use crate::config::PAGE_SIZE;
use crate::task::scheduler::current_proc;
use alloc::vec::Vec;

pub mod inode;
pub mod stdio;

pub trait File: Send + Sync {
    fn read<'a>(&self, buf: &mut UserBuffer) -> usize;
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize;
}

pub struct UserBuffer<'a> {
    array: Vec<&'a mut [u8]>,
    len: usize,
}

impl<'a> UserBuffer<'_> {
    pub fn from_current_proc(addr: usize, len: usize) -> Self {
        let buffer = current_proc().translate_buffer(addr, len);
        let mut len = 0;
        for b in &buffer {
            len += b.len();
        }
        return Self {
            array: buffer,
            len: len,
        };
    }

    #[inline]
    pub fn length(&self) -> usize {
        return self.len;
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) {
        assert!(offset < self.len && offset + data.len() <= self.len);
        let (start_idx, end_idx, start_off, end_off) = self.get_range(offset, data.len());
        let mut l = start_off;
        let mut r;
        let mut data_off = 0;
        for i in start_idx..(end_idx + 1) {
            if i == end_idx {
                r = end_off;
            } else {
                r = self.array[i].len();
            }
            self.array[i][l..r].copy_from_slice(&data[data_off..(data_off + r - l)]);
            data_off += r - l;
            l = 0;
        }
    }

    pub fn read(&self, offset: usize, data: &mut [u8]) {
        assert!(offset < self.len && offset + data.len() <= self.len);
        let (start_idx, end_idx, start_off, end_off) = self.get_range(offset, data.len());
        let mut l = start_off;
        let mut r;
        let mut data_off = 0;
        for i in start_idx..(end_idx + 1) {
            if i == end_idx {
                r = end_off;
            } else {
                r = self.array[i].len();
            }
            data[data_off..(data_off + r - l)].copy_from_slice(&self.array[i][l..r]);
            data_off += r - l;
            l = 0;
        }
    }

    pub fn foreach<F: FnMut(&mut [u8])>(&mut self, mut f: F) {
        for bytes in self.array.iter_mut() {
            f(*bytes);
        }
    }

    fn get_range(&self, offset: usize, len: usize) -> (usize, usize, usize, usize) {
        let end = offset + len;
        let (start_idx, end_idx);
        let (start_off, end_off);
        if offset < self.array[0].len() {
            start_idx = 0;
            start_off = offset;
        } else {
            start_idx = (offset - self.array[0].len()) / PAGE_SIZE + 1;
            start_off = (offset - self.array[0].len()) % PAGE_SIZE;
        }
        if end <= self.array[0].len() {
            end_idx = 0;
            end_off = end;
        } else {
            end_idx = (end - self.array[0].len()) / PAGE_SIZE + 1;
            end_off = (end - self.array[0].len()) % PAGE_SIZE;
        }
        return (start_idx, end_idx, start_off, end_off);
    }
}
