use super::virtio::*;
use array_macro::array;
use crate::config::PAGE_SIZE;
use crate::mem::address::*;
use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;
use crate::arch::riscv::qemu::layout::SECTOR_SIZE;

pub struct VirtIOBlock {
    desc: [VirtQueueDesc; QUEUE_SIZE],
    avail: VirtQueueAvail,
    used: VirtQueueUsed,
    
    free_descs: [bool; QUEUE_SIZE],
    used_idx: u16,
}

impl VirtIOBlock {
    pub fn new() -> Self {
        Self { desc: array![_ => VirtQueueDesc::new(); QUEUE_SIZE], avail: VirtQueueAvail::new(), used: VirtQueueUsed::new(), free_descs: [true; QUEUE_SIZE], used_idx: 0}
    }
    // 初始化驱动过程，see：virtio-v1.1.pdf，3.1.1
    pub unsafe fn init(&self) {
        // check device
        if read(VIRTIO_MMIO_MAGIC_VALUE) != VIRTIO_MAGIC_NUM || read(VIRTIO_MMIO_VERSION) != 1
           || read(VIRTIO_MMIO_DEVICE_ID) != VIRTIO_DEVICE_BLOCK || read(VIRTIO_MMIO_VENDOR_ID) != 0x554d4551
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
        features &= !(1u32 << VIRTIO_BLK_F_RO);
        features &= !(1u32 << VIRTIO_BLK_F_SCSI);
        features &= !(1u32 << VIRTIO_BLK_F_CONFIG_WCE);
        features &= !(1u32 << VIRTIO_BLK_F_MQ);
        features &= !(1u32 << VIRTIO_F_ANY_LAYOUT);
        features &= !(1u32 << VIRTIO_RING_F_EVENT_IDX);
        features &= !(1u32 << VIRTIO_RING_F_INDIRECT_DESC);
        write(VIRTIO_MMIO_DRIVER_FEATURES, features);
        // 设置FEATURES_OK
        // feature协商结束
        status |= VIRTIO_CONFIG_S_FEATURES_OK;
        write(VIRTIO_MMIO_STATUS, status);
        // 设置DRIVER_OK，driver初始化结束
        status |= VIRTIO_CONFIG_S_DRIVER_OK;
        write(VIRTIO_MMIO_STATUS, status);
        
        // 设置page size
        write(VIRTIO_MMIO_GUEST_PAGE_SIZE, PAGE_SIZE as u32);
        
        // 接下来初始化queue 0
        write(VIRTIO_MMIO_QUEUE_SEL, 0);
        // 设置queue size
        let max_queue = read(VIRTIO_MMIO_QUEUE_NUM_MAX);
        if QUEUE_SIZE > max_queue as usize {
            panic!("queue size too large")
        }else {
            write(VIRTIO_MMIO_QUEUE_NUM, QUEUE_SIZE as u32);
        }
        // 设置queue的地址
        let ppn = PhysAddr(self as *const VirtIOBlock as usize).page_number().0;
        write(VIRTIO_MMIO_QUEUE_PFN, ppn as u32);
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
                i =  next as usize;
            }else {
                break;
            }
        }
    }
    
    fn alloc_descs(&mut self, count: usize, idxs: &mut [usize]) {
        assert!(count <= QUEUE_SIZE && count == idxs.len());
        for i in 0..count {
            if let Some(idx) = self.alloc_desc() {
                idxs[i] = idx;
            }else {
                panic!("no enough descs to alloc");
            }
        }
    }
}

impl BlockDevice for VirtIOBlock {
    fn read(&self, block_id: u32, data: &mut [u8]) {
        unsafe {
            // 因为文件系统的块大小和块设备的块大小不同，所以需要将大块拆分
            let sectors = blockid_to_sector(block_id);
            sectors.iter().enumerate().for_each(|(i, sector)| {
                let mut req = VirtIOBlkReq::new();
                req.type_ = VIRTIO_BLK_OP_IN;
                req.sector = *sector as u64;
                // sector块的目标buffer
                let offset = i * SECTOR_SIZE;
                let buf = &mut data[offset..offset + SECTOR_SIZE];

                let mut desc_idxs = [0usize; 2];
                self.alloc_descs(2, &mut desc_idxs);
                self.desc[desc_idxs[0]].addr = &req as *const VirtIOBlkReq as u64;
                self.desc[desc_idxs[0]].len = core::mem::size_of::<VirtIOBlkReq>() as u32;
                self.desc[desc_idxs[0]].flags |= VIRTQ_DESC_FLAG_NEXT;
                self.desc[desc_idxs[0]].next = desc_idxs[1] as u16;
                
                self.desc[desc_idxs[1]].addr = buf.as_ptr() as u64;
                self.desc[desc_idxs[1]].len = buf.len() as u32;
                self.desc[desc_idxs[1]].flags = VIRTQ_DESC_FLAG_WRITE;
                self.desc[desc_idxs[1]].next = 0;
            });
        }
    }
    fn write(&self, block_id: u32, data: &[u8]) {
        
    }
}

fn blockid_to_sector(block_id: u32) -> [u32; BLOCK_SIZE as usize / SECTOR_SIZE] {
    let mut sectors = [0u32; BLOCK_SIZE as usize / SECTOR_SIZE];
    let first = block_id *  (BLOCK_SIZE / SECTOR_SIZE as u32);
    for i in 0..sectors.len() {
        sectors[i] = first + i as u32;
    }
    return sectors;
}

// block device ops
const VIRTIO_BLK_OP_IN: u32 = 0; // read
const VIRTIO_BLK_OP_OUT: u32 = 1; // write

// 块设备IO请求
#[repr(C)]
struct VirtIOBlkReq {
    type_: u32,           // 请求类型：读、写
    reserved: u32,        // 保留位 0
    sector: u64,          // 读取的扇区编号
}

impl VirtIOBlkReq {
    const fn new() -> Self {
        Self {
            type_: 0,
            reserved: 0,
            sector: 0,
        }
    }
}