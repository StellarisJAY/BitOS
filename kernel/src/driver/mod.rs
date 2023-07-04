pub mod blk;
pub mod net;
pub mod pci;
pub mod plic;
pub mod uart;
pub mod virtio;

pub fn init() {
    pci::scan_pci_bus();
    plic::init_plic();
    blk::init_blk();
    net::init();
}
