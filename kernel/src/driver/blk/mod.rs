pub mod virtio_block;

use lazy_static::lazy_static;
use alloc::sync::Arc;
use virtio_block::VirtIOBlock;
use simplefs::block_device::BlockDevice;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = unsafe {
        let virtio_block = VirtIOBlock::new();
        virtio_block.init();
        kernel!("virtio blk init");
        Arc::new(virtio_block)
    };
}

pub fn init_blk() {
    Arc::clone(&BLOCK_DEVICE);
    kernel!("blk device initialized");
}