use crate::arch::riscv::qemu::layout::E1000_REGS;
use crate::config::PAGE_SIZE;
use crate::mem::allocator::{alloc, dealloc, Frame};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::sync::atomic::{fence, Ordering};
use lazy_static::lazy_static;

pub struct E1000Device<'a> {
    rx_ring: &'a mut [RxDesc],
    tx_ring: &'a mut [TxDesc],
    mbuf_size: usize,
    frames: BTreeMap<usize, Frame>,
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

lazy_static! {
    pub static ref E1000_NETWORK_DEV: Arc<E1000Device<'static>> = Arc::new(E1000Device::new());
}

pub fn e1000_init() {
    E1000_NETWORK_DEV.init();
}

impl<'a> E1000Device<'a> {
    pub fn new() -> Self {
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
        };
    }

    pub fn init(&self) {
        // disable interrupt
        write_reg(E1000_IMS, 0);

        // Device RESET
        let ctl: usize = read_reg(E1000_CTL);
        write_reg(E1000_CTL, ctl | E1000_CTL_RST);
        write_reg(E1000_IMS, 1);

        // 初始化TxRing地址，长度
        write_reg(E1000_TDBAL, &self.tx_ring as *const _ as u32);
        write_reg(
            E1000_TDLEN,
            (self.tx_ring.len() * core::mem::size_of::<TxDesc>()) as u32,
        );
        write_reg(E1000_TDT, 0u32);
        write_reg(E1000_TDH, 0u32);
        // 初始化RxRing地址，长度
        write_reg(E1000_RDBAL, &self.rx_ring as *const _ as u32);
        write_reg(
            E1000_RDLEN,
            (self.rx_ring.len() * core::mem::size_of::<RxDesc>()) as u32,
        );
        write_reg(E1000_RDT, 0u32);
        write_reg(E1000_RDH, 0u32);

        //todo more initialization
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

fn write_reg<T: Sized>(offset: usize, val: T) {
    unsafe {
        core::ptr::write_volatile((E1000_REGS + offset) as *mut T, val);
    }
}

fn read_reg<T: Sized>(offset: usize) -> T {
    unsafe { core::ptr::read_volatile((E1000_REGS + offset) as *const _) }
}

const RX_DESC_SPECIAL_OTHER: u16 = 0;
const RX_DESC_SPECIAL_VLAN_MAKS: u16 = (1 << 12) - 1;
const RX_DESC_SPECIAL_PRI_OFF: u16 = 13;
const RX_DESC_SPECIAL_CFI_OFF: u16 = 12;

const TX_RING_SIZE: usize = 256;
const RX_RING_SIZE: usize = 256;
const BUF_SIZE: usize = 2048;

const E1000_CTL: usize = 0x00000;
const E1000_ICR: usize = 0x000C0;
const E1000_IMS: usize = 0x000D0;
const E1000_RCTL: usize = 0x00100;
const E1000_TCTL: usize = 0x00400;
const E1000_TIPG: usize = 0x00410;
const E1000_RDBAL: usize = 0x02800;
const E1000_RDBAH: usize = 0x02804;
const E1000_RDTR: usize = 0x02820;
const E1000_RADV: usize = 0x0282C;
const E1000_RDH: usize = 0x02810;
const E1000_RDT: usize = 0x02818;
const E1000_RDLEN: usize = 0x02808;
const E1000_RSRPD: usize = 0x02C00;
const E1000_TDBAL: usize = 0x03800;
const E1000_TDLEN: usize = 0x03808;
const E1000_TDH: usize = 0x03810;
const E1000_TDT: usize = 0x03818;
const E1000_MTA: usize = 0x05200;
const E1000_RA: usize = 0x05400;

/* Device Control */
const E1000_CTL_SLU: usize = 0x00000040;
const E1000_CTL_FRCSPD: usize = 0x00000800;
const E1000_CTL_FRCDPLX: usize = 0x00001000;
const E1000_CTL_RST: usize = 0x00400000;

/* Transmit Control */
const E1000_TCTL_RST: usize = 0x00000001;
const E1000_TCTL_EN: usize = 0x00000002;
const E1000_TCTL_BCE: usize = 0x00000004;
const E1000_TCTL_PSP: usize = 0x00000008;
const E1000_TCTL_CT: usize = 0x00000ff0;
const E1000_TCTL_CT_SHIFT: usize = 4;
const E1000_TCTL_COLD: usize = 0x003ff000;
const E1000_TCTL_COLD_SHIFT: usize = 12;
const E1000_TCTL_SWXOFF: usize = 0x00400000;
const E1000_TCTL_PBE: usize = 0x00800000;
const E1000_TCTL_RTLC: usize = 0x01000000;
const E1000_TCTL_NRTU: usize = 0x02000000;
const E1000_TCTL_MULR: usize = 0x10000000;

/* Receive Control */
const E1000_RCTL_RST: usize = 0x00000001;
const E1000_RCTL_EN: usize = 0x00000002;
const E1000_RCTL_SBP: usize = 0x00000004;
const E1000_RCTL_UPE: usize = 0x00000008;
const E1000_RCTL_MPE: usize = 0x00000010;
const E1000_RCTL_LPE: usize = 0x00000020;
const E1000_RCTL_LBM_NO: usize = 0x00000000;
const E1000_RCTL_LBM_MAC: usize = 0x00000040;
const E1000_RCTL_LBM_SLP: usize = 0x00000080;
const E1000_RCTL_LBM_TCVR: usize = 0x000000C0;
const E1000_RCTL_DTYP_MASK: usize = 0x00000C00;
const E1000_RCTL_DTYP_PS: usize = 0x00000400;
const E1000_RCTL_RDMTS_HALF: usize = 0x00000000;
const E1000_RCTL_RDMTS_QUAT: usize = 0x00000100;
const E1000_RCTL_RDMTS_EIGTH: usize = 0x00000200;
const E1000_RCTL_MO_SHIFT: usize = 12;
const E1000_RCTL_MO_0: usize = 0x00000000;
const E1000_RCTL_MO_1: usize = 0x00001000;
const E1000_RCTL_MO_2: usize = 0x00002000;
const E1000_RCTL_MO_3: usize = 0x00003000;
const E1000_RCTL_MDR: usize = 0x00004000;
const E1000_RCTL_BAM: usize = 0x00008000;
/* these buffer sizes are valid if E1000_RCTL_BSEX is 0 */
const E1000_RCTL_SZ_2048: usize = 0x00000000; /* rx buffer size 2048 */
const E1000_RCTL_SZ_1024: usize = 0x00010000; /* rx buffer size 1024 */
const E1000_RCTL_SZ_512: usize = 0x00020000; /* rx buffer size 512 */
const E1000_RCTL_SZ_256: usize = 0x00030000; /* rx buffer size 256 */

/* these buffer sizes are valid if E1000_RCTL_BSEX is 1 */
const E1000_RCTL_SZ_16384: usize = 0x00010000; /* rx buffer size 16384 */
const E1000_RCTL_SZ_8192: usize = 0x00020000; /* rx buffer size 8192 */
const E1000_RCTL_SZ_4096: usize = 0x00030000; /* rx buffer size 4096 */
const E1000_RCTL_VFE: usize = 0x00040000; /* vlan filter enable */
const E1000_RCTL_CFIEN: usize = 0x00080000; /* canonical form enable */
const E1000_RCTL_CFI: usize = 0x00100000; /* canonical form indicator */
const E1000_RCTL_DPF: usize = 0x00400000; /* discard pause frames */
const E1000_RCTL_PMCF: usize = 0x00800000; /* pass MAC control frames */
const E1000_RCTL_BSEX: usize = 0x02000000; /* Buffer size extension */
const E1000_RCTL_SECRC: usize = 0x04000000; /* Strip Ethernet CRC */
const E1000_RCTL_FLXBUF_MASK: usize = 0x78000000; /* Flexible buffer size */
const E1000_RCTL_FLXBUF_SHIFT: usize = 27; /* Flexible buffer shift */

const DATA_MAX: usize = 1518;

/* Transmit Descriptor command definitions [E1000 3.3.3.1] */
const E1000_TXD_CMD_EOP: usize = 0x01; /* End of Packet */
const E1000_TXD_CMD_RS: usize = 0x08; /* Report Status */

/* Transmit Descriptor status definitions [E1000 3.3.3.2] */
const E1000_TXD_STAT_DD: u8 = 0x00000001; /* Descriptor Done */
