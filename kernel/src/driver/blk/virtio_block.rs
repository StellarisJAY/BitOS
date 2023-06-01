use crate::driver::virtio::*;
use crate::arch::riscv::qemu::layout::SECTOR_SIZE;
use crate::config::PAGE_SIZE;
use crate::mem::address::*;
use alloc::sync::Arc;
use array_macro::array;
use lazy_static::lazy_static;
use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;
use spin::Mutex;

pub struct VirtIOBlock {
    queue: Mutex<VirtQueue>,
}

lazy_static! {
    static ref VIRTIO_BLOCK: VirtIOBlock = VirtIOBlock::new();
}

impl VirtIOBlock {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VirtQueue::new()),
        }
    }

    pub unsafe fn init(&self) {
        self.queue
            .lock()
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
    
    pub fn read_block(&self, block_id: u32, data: &mut [u8]) {
        assert_eq!(data.len(), SECTOR_SIZE, "read blk buffer must be 512 Bytes");
        let sector = blockid_to_sector_offset(block_id);
        let mut req = VirtIOBlkReq{
            type_: VIRTIO_BLK_OP_IN,
            reserved: 0,
            sector: sector as u64,
        };
        let mut resp = VirtIOBlkResp::new();
        let inputs = [req.as_bytes(), data, resp.as_bytes()];
        let writes = [false, true, true];

        unsafe {
            let mut queue = self.queue.lock();
            let token = queue.add(&inputs, &writes).unwrap();
            queue.notify(0);
            kernel!("waiting respone, avail: {}", queue.avail.idx);
            while !queue.can_pop() {}
            kernel!("response recv");
        }
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
}

#[repr(C)]
struct VirtIOBlkResp {
    status: u8,
}

impl VirtIOBlkReq {
    const fn new() -> Self {
        Self {
            type_: 0,
            reserved: 0,
            sector: 0,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let addr = self as *const _ as usize;
            core::slice::from_raw_parts(addr as *const u8, 16)
        }
    }
}

impl VirtIOBlkResp {
    const fn new() -> Self {
        Self{status: 0xff}
    }
    
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let addr = self as *const _ as usize;
            core::slice::from_raw_parts(addr as *const u8, 1)
        }
    }
}
