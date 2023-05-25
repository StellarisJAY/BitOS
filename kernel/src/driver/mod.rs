pub mod uart;
pub mod virtio;
pub mod virtio_block;

pub fn init() {
    virtio_block::init_block_device();
}
