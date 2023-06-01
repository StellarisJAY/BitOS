use crate::arch::riscv::qemu::layout::VIRTIO0;
use crate::config::PAGE_SIZE;
use crate::mem::address::*;
use alloc::vec::Vec;
use array_macro::array;
use core::sync::atomic::{fence, Ordering};

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
    pub addr: u64,  // buffer地址
    pub len: u32,   // buffer长度
    pub flags: u16, // flags
    pub next: u16,  // 连续buffer的下一个Desc
}

// virtq_avail，可用队列
#[repr(C)]
pub struct VirtQueueAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; QUEUE_SIZE],
}

// UsedElem，已经处理完的buffer
#[repr(C)]
pub struct VirtQueueUsedElem {
    pub id: u32,  // buffer的desc链的第一个desc的id
    pub len: u32, // buffer的长度
}

// VirtQueueUsed, vq_used在device处理完buffer后，将buffer返回。
// driver只能读取vq_used，device只能写
#[repr(C)]
pub struct VirtQueueUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtQueueUsedElem; QUEUE_SIZE],
}

#[repr(C, align(4096))]
pub struct VirtQueue {
    pub desc: [VirtQueueDesc; QUEUE_SIZE],
    pub avail: VirtQueueAvail,
    pub used: VirtQueueUsed,
    free_descs: [bool; QUEUE_SIZE],
    pub used_idx: u16,
}

impl VirtQueue {
    pub fn new() -> Self {
        Self {
            desc: array![_ => VirtQueueDesc::new(); QUEUE_SIZE],
            avail: VirtQueueAvail::new(),
            used: VirtQueueUsed::new(),
            free_descs: [true; QUEUE_SIZE],
            used_idx: 0,
        }
    }
    // 初始化驱动过程，see：virtio-v1.1.pdf，3.1.1
    pub unsafe fn init<F: FnOnce(u32) -> u32>(
        &self,
        device_type: u32,
        queue_sel: u32,
        set_features: F,
    ) {
        // check device
        if read(VIRTIO_MMIO_MAGIC_VALUE) != VIRTIO_MAGIC_NUM
            || read(VIRTIO_MMIO_VERSION) != 2
            || read(VIRTIO_MMIO_DEVICE_ID) != device_type
            || read(VIRTIO_MMIO_VENDOR_ID) != 0x554d4551
        {
            panic!("can not find virtio block device");
        }
        // 设置ACKNOWLEDGE
        let mut status: u32 = 0;
        status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
        write(VIRTIO_MMIO_STATUS, status);
        // 设置DRIVER
        status |= VIRTIO_CONFIG_S_DRIVER;
        write(VIRTIO_MMIO_STATUS, status);

        // 读取features bits
        let mut features: u32 = read(VIRTIO_MMIO_DEVICE_FEATURES);
        // 修改features，协商driver需要的features
        features = set_features(features);
        write(VIRTIO_MMIO_DRIVER_FEATURES, features);

        status |= VIRTIO_CONFIG_S_FEATURES_OK;
        write(VIRTIO_MMIO_STATUS, status);
        // 设置DRIVER_OK，driver初始化结束
        status |= VIRTIO_CONFIG_S_DRIVER_OK;
        write(VIRTIO_MMIO_STATUS, status);

        // 配置当前virt_queue, see: virtio-v1.1.pdf，4.2.3.2
        write(VIRTIO_MMIO_QUEUE_SEL, queue_sel);
        if read(VIRTIO_MMIO_QUEUE_READY) != 0 {
            panic!("virt queue already initialized");
        }
        // 设置queue size
        let max_queue = read(VIRTIO_MMIO_QUEUE_NUM_MAX);
        if QUEUE_SIZE > max_queue as usize {
            panic!("queue size too large")
        } else {
            write(VIRTIO_MMIO_QUEUE_NUM, QUEUE_SIZE as u32);
        }
        // 写入desc、driver、device的访问地址
        let desc_addr = &self.desc as *const VirtQueueDesc as usize;
        let avail_addr = &self.avail as *const VirtQueueAvail as usize;
        let used_addr = &self.used as *const VirtQueueUsed as usize;
        write(VIRTIO_MMIO_QUEUE_DESC_LOW, desc_addr as u32);
        write(VIRTIO_MMIO_QUEUE_DESC_HIGH, (desc_addr >> 32) as u32);
        write(VIRTIO_MMIO_QUEUE_DRIVER_LOW, avail_addr as u32);
        write(VIRTIO_MMIO_QUEUE_DRIVER_HIGH, (avail_addr >> 32) as u32);
        write(VIRTIO_MMIO_QUEUE_DEVICE_LOW, used_addr as u32);
        write(VIRTIO_MMIO_QUEUE_DEVICE_HIGH, (used_addr >> 32) as u32);
        write(VIRTIO_MMIO_QUEUE_READY, 1);
        fence(Ordering::SeqCst);
    }

    pub unsafe fn add(&mut self, inputs: &[&[u8]], writes: &[bool]) -> Option<u16> {
        if inputs.is_empty() {
            return None;
        }
        let descs = self.alloc_descs(inputs.len());
        let head = descs[0];
        // inputs转换成desc链
        inputs.iter().enumerate().for_each(|(i, buf)| {
            let idx = descs[i] as usize;
            self.desc[idx].addr = (*buf).as_ptr() as u64;
            self.desc[idx].len = (*buf).len() as u32;
            if writes[i] {
                self.desc[idx].flags = VIRTQ_DESC_FLAG_WRITE;
            }
            if i == inputs.len() - 1 {
                self.desc[idx].next = 0;
            } else {
                self.desc[idx].flags |= VIRTQ_DESC_FLAG_NEXT;
                self.desc[idx].next = descs[i + 1];
            }
        });

        // desc链放入avail vring
        let avail_idx = self.avail.idx as usize % QUEUE_SIZE;
        self.avail.ring[avail_idx] = head;
        fence(Ordering::SeqCst);
        // 增加avail idx
        self.avail.idx += 1;
        fence(Ordering::SeqCst);
        Some(head)
    }

    pub unsafe fn notify(&self, sel: u32) {
        write(VIRTIO_MMIO_QUEUE_NOTIFY, sel);
    }

    pub fn can_pop(&self) -> bool {
        fence(Ordering::SeqCst);
        return self.used.idx != self.used_idx;
    }

    fn alloc_desc(&mut self) -> Option<usize> {
        for i in 0..QUEUE_SIZE {
            if self.free_descs[i] {
                self.free_descs[i] = false;
                return Some(i);
            }
        }
        return None;
    }

    fn dealloc_desc(&mut self, i: usize) {
        self.desc[i].addr = 0;
        self.desc[i].len = 0;
        self.desc[i].flags = 0;
        self.desc[i].next = 0;
        self.free_descs[i] = true;
    }

    fn free_desc_chain(&mut self, first: usize) {
        let mut i = first;
        loop {
            let flags = self.desc[i].flags;
            let next = self.desc[i].next;
            self.dealloc_desc(i);
            // 是否还有下一个desc
            if (flags & VIRTQ_DESC_FLAG_NEXT) != 0 {
                i = next as usize;
            } else {
                break;
            }
        }
    }

    fn alloc_descs(&mut self, count: usize) -> Vec<u16> {
        let mut res: Vec<u16> = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(idx) = self.alloc_desc() {
                res.push(idx as u16);
            } else {
                panic!("no enough descs to alloc");
            }
        }
        return res;
    }
}

impl VirtQueueDesc {
    pub fn new() -> Self {
        Self {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

impl VirtQueueAvail {
    pub fn new() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [0; QUEUE_SIZE],
        }
    }
}

impl VirtQueueUsed {
    pub fn new() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: array![_ => VirtQueueUsedElem::new(); QUEUE_SIZE],
        }
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

const VIRTIO_MMIO_QUEUE_DESC_LOW: usize = 0x080;
const VIRTIO_MMIO_QUEUE_DESC_HIGH: usize = 0x084;
const VIRTIO_MMIO_QUEUE_DRIVER_LOW: usize = 0x090;
const VIRTIO_MMIO_QUEUE_DRIVER_HIGH: usize = 0x094;
const VIRTIO_MMIO_QUEUE_DEVICE_LOW: usize = 0x0a0;
const VIRTIO_MMIO_QUEUE_DEVICE_HIGH: usize = 0x0a4;
