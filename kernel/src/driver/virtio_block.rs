use super::virtio::*;
use crate::arch::riscv::qemu::layout::SECTOR_SIZE;
use crate::config::PAGE_SIZE;
use crate::mem::address::*;
use crate::sync::cell::SafeCell;
use alloc::sync::Arc;
use array_macro::array;
use lazy_static::lazy_static;
use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;

pub struct VirtIOBlock {
    queue: SafeCell<VirtQueue>,
}

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = unsafe {
        let virtio_block = VirtIOBlock::new();
        virtio_block.init();
        Arc::new(virtio_block)
    };
}

lazy_static! {
    static ref VIRTIO_BLOCK: VirtIOBlock = VirtIOBlock::new();
}

pub fn init_block_device() {
    let _ = Arc::clone(&BLOCK_DEVICE);
    kernel!("VirtIO Blk driver initialized");
}

impl VirtIOBlock {
    pub fn new() -> Self {
        Self {
            queue: SafeCell::new(VirtQueue::new()),
        }
    }

    pub unsafe fn init(&self) {
        self.queue
            .borrow()
            .init(VIRTIO_DEVICE_BLOCK, 0, |mut features| {
                features &= !(1u32 << VIRTIO_BLK_F_RO);
                features &= !(1u32 << VIRTIO_BLK_F_SCSI);
                features &= !(1u32 << VIRTIO_BLK_F_CONFIG_WCE);
                features &= !(1u32 << VIRTIO_BLK_F_MQ);
                features &= !(1u32 << VIRTIO_F_ANY_LAYOUT);
                features &= !(1u32 << VIRTIO_RING_F_EVENT_IDX);
                features &= !(1u32 << VIRTIO_RING_F_INDIRECT_DESC);
                return features;
            });
    }
}

impl BlockDevice for VirtIOBlock {
    fn read(&self, block_id: u32, data: &mut [u8]) {
        // todo read
    }
    fn write(&self, block_id: u32, data: &[u8]) {
        // todo write
    }
}

#[inline]
fn blockid_to_sector_offset(block_id: u32) -> u32 {
    return block_id * (BLOCK_SIZE / SECTOR_SIZE as u32);
}

// block device ops
const VIRTIO_BLK_OP_IN: u32 = 0; // read
const VIRTIO_BLK_OP_OUT: u32 = 1; // write

// 块设备IO请求
#[repr(C)]
struct VirtIOBlkReq {
    type_: u32,    // 请求类型：读、写
    reserved: u32, // 保留位 0
    sector: u64,   // 读取的扇区编号
    data: [u8; SECTOR_SIZE],
    status: u8,
}

impl VirtIOBlkReq {
    const fn new() -> Self {
        Self {
            type_: 0,
            reserved: 0,
            sector: 0,
            data: [0u8; SECTOR_SIZE],
            status: 0,
        }
    }
}
