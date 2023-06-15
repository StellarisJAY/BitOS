pub mod mem_block;
pub mod virtio_block;

use crate::config::{BlockDeviceType, BLOCK_DEVICE_TYPE};
use alloc::sync::Arc;
use lazy_static::lazy_static;
use mem_block::MemoryBlockDevice;
use simplefs::block_device::BlockDevice;
use virtio_block::VirtIOBlock;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = unsafe {
        let block_device: Arc<dyn BlockDevice>;
        match BLOCK_DEVICE_TYPE {
            BlockDeviceType::MEMORY => block_device = Arc::new(MemoryBlockDevice::new()),
            BlockDeviceType::VIRTIO => {
                panic!("virtio block not available");
                let virtio_block = VirtIOBlock::new();
                virtio_block.init();
                block_device = Arc::new(virtio_block);
            }
        }
        block_device
    };
}

pub fn init_blk() {
    Arc::clone(&BLOCK_DEVICE);
    kernel!("blk device initialized");
}
