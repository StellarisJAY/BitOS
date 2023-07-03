use crate::config::PAGE_SIZE;
use crate::mem::allocator::{alloc, dealloc, Frame};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::sync::atomic::{fence, Ordering};

pub struct E1000Device<'a> {
    rx_ring: &'a mut [RxDesc],
    tx_ring: &'a mut [TxDesc],
    mbuf_size: usize,
    frames: BTreeMap<usize, Frame>,
    mmio_base: usize,
}

// RecieveDescriptor 接收网络包的描述
#[derive(Debug, Clone)]
#[repr(C)]
pub struct RxDesc {
    buffer: u64, // buffer地址
    length: u16, // buffer长度
    checksum: u16,
    status: u8, // 状态码，see DescSaatus
    error: u8,
    special: u16,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TxDesc {
    buffer: u64,
    length: u16,
    checksum_offset: u8,
    cmd: u8,
    status: u8,
    checksum_start: u8,
    special: u16,
}

bitflags! {
    pub struct RxDescStatus: u8 {
        const DD =  1<<0;       // Descriptor Done, 硬件已经处理完了Descriptor
        const EOP = 1<<1;       // End Of Packet，多个buffer组成网络包的最后一个buffer
        const IXSM = 1<<2;      // 忽略Tcp和Ip checksum
        const VP = 1<<3;        // VLAN Packet
        const RSV = 1<<4;       // Reserved always 0
        const TCPCS = 1<<5;     // TCP Checksum
        const IPCS = 1<<6;      // IP Checksum
        const PIF = 1<<7;       // 是否通过filter
    }
}

bitflags! {
    pub struct RxDescError: u8 {
        const CE = 1 << 0;
        const SE_RSV = 1 << 1;
        const CE_RSV = 1 << 2;
        const RSV = 1 << 3;
        const RSV_CXE = 1 << 4;
        const TCPE = 1 << 5;     // TCP checksum error
        const IPE = 1 << 6;      // IP  checksum error
        const RXE = 1 << 7;      // Resv packet error
    }
}

impl<'a> E1000Device<'a> {
    pub fn new(mmio_addr: usize) -> Self {
        // 分配tx和rx ring的物理内存页
        let tx_ring_pages =
            (TX_RING_SIZE * core::mem::size_of::<TxDesc>() + PAGE_SIZE - 1) / PAGE_SIZE;
        let rx_ring_pages =
            (RX_RING_SIZE * core::mem::size_of::<RxDesc>() + PAGE_SIZE - 1) / PAGE_SIZE;
        let mut frames: BTreeMap<usize, Frame> = BTreeMap::new();

        let tx_ring = unsafe {
            core::slice::from_raw_parts_mut(
                alloc_frames(&mut frames, tx_ring_pages) as *mut TxDesc,
                TX_RING_SIZE,
            )
        };
        let rx_ring = unsafe {
            core::slice::from_raw_parts_mut(
                alloc_frames(&mut frames, rx_ring_pages) as *mut RxDesc,
                RX_RING_SIZE,
            )
        };
        // 分配tx和rx 的buffers
        let tx_buf_pages = (TX_RING_SIZE * BUF_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
        let rx_buf_pages = (RX_RING_SIZE * BUF_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;

        let mut tx_buf_addr = alloc_frames(&mut frames, tx_buf_pages);
        for i in 0..tx_buf_pages {
            tx_ring[i].buffer = tx_buf_addr as u64;
            tx_buf_addr += BUF_SIZE;
        }
        let mut rx_buf_addr = alloc_frames(&mut frames, rx_buf_pages);
        for i in 0..rx_buf_pages {
            tx_ring[i].buffer = rx_buf_addr as u64;
            rx_buf_addr += BUF_SIZE;
        }

        return Self {
            rx_ring: rx_ring,
            tx_ring: tx_ring,
            mbuf_size: BUF_SIZE,
            frames: frames,
            mmio_base: mmio_addr,
        };
    }

    pub fn init(&self) {}

    fn read_reg(&self, offset: usize) -> u32 {
        let ptr = (self.mmio_base + offset) as *const u32;
        unsafe { ptr.read_volatile() }
    }

    fn write_reg(&self, offset: usize, val: u32) {
        let ptr = (self.mmio_base + offset) as *mut u32;
        unsafe {
            ptr.write_volatile(val);
        }
    }
}

fn alloc_frames(frames: &mut BTreeMap<usize, Frame>, pages: usize) -> usize {
    let mut first_page: Option<usize> = None;
    for _ in 0..pages {
        let frame = alloc().unwrap();
        if let None = first_page {
            first_page = Some(frame.ppn.base_addr())
        }
        frames.insert(frame.ppn.0, frame);
    }
    return first_page.unwrap();
}

const RX_DESC_SPECIAL_OTHER: u16 = 0;
const RX_DESC_SPECIAL_VLAN_MAKS: u16 = (1 << 12) - 1;
const RX_DESC_SPECIAL_PRI_OFF: u16 = 13;
const RX_DESC_SPECIAL_CFI_OFF: u16 = 12;

const TX_RING_SIZE: usize = 256;
const RX_RING_SIZE: usize = 256;
const BUF_SIZE: usize = 2048;
