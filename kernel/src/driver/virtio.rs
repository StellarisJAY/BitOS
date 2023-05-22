use crate::arch::riscv::qemu::layout::VIRTIO0;
use array_macro::array;

// virtio driver，部分功能没有实现。
// virtio规范，see: https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf

pub const QUEUE_SIZE: usize = 8;
// buffer是否还有连续的下一个部分
pub const VIRTQ_DESC_FLAG_NEXT: u16 = 1;
// 标记buffer对device是writeonly或readonly
pub const VIRTQ_DESC_FLAG_WRITE: u16 = 2;

// VirtQueueDesc 描述一个virtq buffer
#[repr(C)]
pub struct VirtQueueDesc {
    pub addr: u64,              // buffer地址
    pub len: u32,               // buffer长度
    pub flags: u16,             // flags
    pub next: u16,              // 连续buffer的下一个Desc
}

// virtq_avail，可用队列
pub struct VirtQueueAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; QUEUE_SIZE],
    pub used_event: u16,
}

// UsedElem，已经处理完的buffer
#[repr(C)]
pub struct VirtQueueUsedElem {
    pub id: u32,      // buffer的desc链的第一个desc的id
    pub len: u32,     // buffer的长度
}

// VirtQueueUsed, vq_used在device处理完buffer后，将buffer返回。
// driver只能读取vq_used，device只能写
pub struct VirtQueueUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtQueueUsedElem; QUEUE_SIZE],
    pub avail_event: u16,
}

impl VirtQueueDesc {
    pub fn new() -> Self {
        Self{
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

impl VirtQueueAvail {
    pub fn new() -> Self {
        Self { flags: 0, idx: 0, ring: [0; QUEUE_SIZE], used_event: 0 }
    }
}

impl VirtQueueUsed {
    pub fn new() -> Self {
        Self { flags: 0, idx: 0, ring: array![_ => VirtQueueUsedElem::new(); QUEUE_SIZE], avail_event: 0 }
    }
}

impl VirtQueueUsedElem {
    pub fn new() -> Self {
        Self { id: 0, len: 0 }
    }
}

#[inline]
pub unsafe fn read(offset: usize) -> u32 {
    let ptr = (VIRTIO0 + offset) as *const u32;
    return ptr.read_volatile();
}

#[inline]
pub unsafe fn write(offset: usize, value: u32) {
    let ptr = (VIRTIO0 + offset) as *mut u32;
    ptr.write_volatile(value);
}

pub const VIRTIO_MAGIC_NUM: u32 = 0x74726976;

// virtio mmio 寄存器地址，see：virtio-v1.1.pdf 4.2.2
// from qemu's virtio_mmio.h
pub const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
pub const VIRTIO_MMIO_VERSION: usize = 0x004;
pub const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
pub const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
pub const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
pub const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
pub const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x028;
pub const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
pub const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
pub const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
pub const VIRTIO_MMIO_QUEUE_ALIGN: usize = 0x03c;
pub const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
pub const VIRTIO_MMIO_QUEUE_READY: usize = 0x044; 
pub const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
pub const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060;
pub const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064;
pub const VIRTIO_MMIO_STATUS: usize = 0x070;

// config bits
pub const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
pub const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
pub const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
pub const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 8;

// device feature bits
pub const VIRTIO_BLK_F_RO: u8 = 5;
pub const VIRTIO_BLK_F_SCSI: u8 = 7;
pub const VIRTIO_BLK_F_CONFIG_WCE: u8 = 11;
pub const VIRTIO_BLK_F_MQ: u8 = 12;
pub const VIRTIO_F_ANY_LAYOUT: u8 = 27;
pub const VIRTIO_RING_F_INDIRECT_DESC: u8 = 28;
pub const VIRTIO_RING_F_EVENT_IDX: u8 = 29;

pub const VIRTIO_DEVICE_NETWORK: u32 = 1;
pub const VIRTIO_DEVICE_BLOCK: u32 = 2;

