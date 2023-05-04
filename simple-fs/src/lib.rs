extern crate alloc;

pub mod bitmap;
pub mod block_cache;
pub mod block_device;
pub mod layout;
pub mod inode;

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_bitmap_compose_and_decompose() {
        bitmap::test_compose_and_decompose();
    }

    #[test]
    fn test_blocks_for_size() {
        inode::test_blocks_for_size();
    }
}