use crate::arch::riscv::qemu::layout::PCIE0;

const PCIE_HEADER_TYPE_EP: u8 = 0;
const PCIE_HEADER_TYPE_BRIDGE: u8 = 1;

#[derive(Clone, Debug)]
#[repr(C)]
struct PcieSharedHeader {
    vendor: u16,
    device: u16,
    command: u16,
    status: u16,
    revision_and_class: u32,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct PcieEndpointHeader {
    shared: PcieSharedHeader,
    base_address_registers: [u32; 6],
    card_bus_cis_ptr: u32,
    sub_vendor: u16,
    sub_id: u16,
    expansion_rom_base_addr: u32,
    capability_ptr: u8,
    reserved: [u8; 7],
    interrupt_line: u8,
    interrupt_pin: u8,
    min_gnt: u8,
    max_lat: u8,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct PcieBridgeHeader {
    shared: PcieSharedHeader,
}

pub fn scan_pci_bus() {
    for dev in 0..32 {
        let off = bdf(0, dev, 0) as usize;
        let base = PCIE0 + off * 4;
        let vendor: u16 = read(base + 0x0);
        let device: u16 = read(base + 0x2);
        // invalid vendor
        if vendor == 0xffff || device == 0xffff {
            continue;
        }
        let header_type: u8 = read(base + 96 + 16);
        // skip bridge
        if header_type == PCIE_HEADER_TYPE_BRIDGE {
            continue;
        }
        let header: PcieEndpointHeader = read(base);
        kernel!(
            "[PCI Bus0] found device, vendor: {:#x}, id: {:#x}, interrupt pin: {}",
            header.shared.vendor,
            header.shared.device,
            header.interrupt_pin
        );
    }
}

fn bdf(bus: u32, device: u32, func: u32) -> u32 {
    return (bus << 16) | (device << 11) | (func << 8);
}

fn read<T: Sized>(addr: usize) -> T {
    unsafe { (addr as *const T).read_volatile() }
}

fn write<T: Sized>(addr: usize, val: T) {
    unsafe {
        (addr as *mut T).write_volatile(val);
    }
}
