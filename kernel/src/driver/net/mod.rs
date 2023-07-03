pub mod e1000;

use crate::arch::riscv::qemu::layout::E1000_BASE;
use alloc::sync::Arc;
use e1000::E1000Device;
use elf::abi::DT_GUILE_VM_VERSION;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NETWORK_DEV: Arc<E1000Device<'static>> = {
        kernel!("creating e1000 network device");
        let dev = E1000Device::new(E1000_BASE);
        dev.init();
        Arc::new(dev)
    };
}

pub fn init() {
    Arc::clone(&NETWORK_DEV);
    kernel!("network device initialized");
}
