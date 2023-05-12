use virtio_drivers::Hal;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use crate::mem::allocator::{alloc, dealloc, Frame};
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;
use crate::mem::address::{PhysPageNumber, PhysAddr};
use simplefs::block_device::BlockDevice;
use alloc::sync::Arc;

const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock(Mutex<VirtIOBlk<HalImpl, MmioTransport>>);

struct HalImpl ();

lazy_static! {
    static ref QUEUED_FRAMES: Mutex<Vec<Frame>> = Mutex::new(Vec::new());
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(VirtIOBlock::new());
}

unsafe impl Send for VirtIOBlock {}
unsafe impl Sync for VirtIOBlock {}

impl VirtIOBlock {
    pub fn new() -> Self {
        unsafe {
            let transport = MmioTransport::new(core::ptr::NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap()).unwrap();
            Self(Mutex::new(VirtIOBlk::new(transport).unwrap()))
        }
    }
}


impl BlockDevice for VirtIOBlock {
    fn read(&self, block_id: u32, data: &mut [u8]) {
        self.0.lock().read_block(block_id as usize, data).expect("error when reading virtio block");
    }
    
    fn write(&self, block_id: u32, data: &[u8]) {
        self.0.lock().write_block(block_id as usize, data).expect("error when writing virtio block");
    }
}

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, direction: virtio_drivers::BufferDirection) -> (virtio_drivers::PhysAddr, core::ptr::NonNull<u8>) {
        let mut frames = QUEUED_FRAMES.lock();
        let mut base_ppn = PhysPageNumber(0); 
        for i in 0..pages {
            let frame = alloc().unwrap();
            if i == 0 {
                base_ppn = frame.ppn;
            }
            assert!(base_ppn.0 + i == frame.ppn.0);
            frames.push(frame);
        }
        return (base_ppn.base_addr(), core::ptr::NonNull::new(base_ppn.base_addr() as _).unwrap());
    }
    unsafe fn dma_dealloc(paddr: virtio_drivers::PhysAddr, vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        let mut ppn = PhysAddr(paddr).page_number();
        for i in 0..pages {
            dealloc(ppn);
            ppn.0 += 1;
        }
        return 0;
    }

    unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, size: usize) -> core::ptr::NonNull<u8> {
        core::ptr::NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: core::ptr::NonNull<[u8]>, direction: virtio_drivers::BufferDirection) -> virtio_drivers::PhysAddr {
        // 由于直接映射，va=pa
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        return vaddr;
    }

    unsafe fn unshare(paddr: virtio_drivers::PhysAddr, buffer: core::ptr::NonNull<[u8]>, direction: virtio_drivers::BufferDirection) {

    }
}