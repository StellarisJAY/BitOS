extern crate alloc;

pub mod bitmap;
pub mod block_cache;
pub mod block_device;
pub mod inode;
pub mod super_block;
pub mod simple_fs;

pub mod layout {
    // 一个磁盘块的大小：4KiB
    pub const BLOCK_SIZE: u32 = 4096;
}