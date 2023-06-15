pub mod blk;
pub mod uart;
pub mod virtio;

pub fn init() {
    blk::init_blk();
}
