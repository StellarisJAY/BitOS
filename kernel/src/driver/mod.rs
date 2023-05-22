pub mod uart;
pub mod virtio_block;
pub mod virtio;

pub fn init() {
    uart::Uart::init();
}
