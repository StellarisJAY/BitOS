use alloc::vec::Vec;

pub mod inode;

pub trait File: Send + Sync {
    fn read<'a>(&self, buf: Vec<&'a mut [u8]>) -> usize;
    fn write<'a>(&self, buf: Vec<&'a [u8]>) -> usize;
}
