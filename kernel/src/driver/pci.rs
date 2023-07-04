use super::net::e1000::e1000_init;
use crate::arch::riscv::qemu::layout::{E1000_REGS, PCIE0};
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

const PCI_HEADER_TYPE_EP: u8 = 0;
const PCI_HEADER_TYPE_BRIDGE: u8 = 1;

#[derive(Clone, Debug)]
#[repr(C)]
struct PCIHeader {
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
struct PCIEndpointHeader {
    shared: PCIHeader,
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
struct PCIBridgeHeader {
    shared: PCIHeader,
}

// 扫描pci总线，扫描到可用设备后对设备驱动初始化
pub fn scan_pci_bus() {
    for dev in 0..32 {
        let off = bdf(0, dev, 0) as usize;
        let base = PCIE0 + off * 4;
        let vendor: u16 = read(base + 0x0);
        let device: u16 = read(base + 0x2);
        // 无效设备
        if vendor == 0xffff || device == 0xffff {
            continue;
        }
        let header_type: u8 = read(base + 96 + 16);
        // 暂时不处理桥接设备
        if header_type == PCI_HEADER_TYPE_BRIDGE {
            continue;
        }
        let header: PCIEndpointHeader = read(base);
        kernel!(
            "[PCI Bus0] found device, vendor: {:#x}, id: {:#x}, interrupt pin: {}, cap: {:#x}",
            header.shared.vendor,
            header.shared.device,
            header.interrupt_pin,
            header.capability_ptr,
        );

        init_device(vendor, device, base);
    }
}

fn init_device(vendor: u16, device: u16, base: usize) {
    match (vendor, device) {
        // e1000 device
        (0x8086, 0x100e) | (0x8086, 0x100f) | (0x8086, 0x10d3) => {
            unsafe {
                // write cmd, io space/mem space/mastering
                ptr::write_volatile((base + size_of::<u32>()) as *mut u16, 7);
                fence(Ordering::SeqCst);
                for i in 0..6 {
                    let addr = base + (4 + i) * size_of::<u32>();
                    let val = ptr::read_volatile(addr as *const u32);
                    ptr::write_volatile(addr as *mut u32, 0xffffffff);
                    fence(Ordering::SeqCst);
                    ptr::write_volatile(addr as *mut u32, val);
                }
                // BAR[0]设置成E1000的各个寄存器基址
                ptr::write_volatile((base + 4 * size_of::<u32>()) as *mut u32, E1000_REGS as u32);
                e1000_init();
            }
        }
        _ => {}
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

const PCI_HEADER_COMMAND: usize = 4;
