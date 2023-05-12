pub mod uart;
pub mod virtio_blk;

pub fn init() {
    uart::Uart::init();
}
