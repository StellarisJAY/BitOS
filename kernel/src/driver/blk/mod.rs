pub mod virtio_block;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use simplefs::block_device::BlockDevice;
use virtio_block::VirtIOBlock;

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
