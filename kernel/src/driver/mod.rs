pub mod blk;
pub mod plic;
pub mod uart;
pub mod virtio;

pub fn init() {
    plic::init_plic();
    blk::init_blk();
}
