pub mod uart;
pub mod virtio;
pub mod blk;

pub fn init() {
    blk::init_blk();
}
