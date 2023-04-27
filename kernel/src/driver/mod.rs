pub mod uart;

pub fn init() {
    uart::Uart::init();
}
