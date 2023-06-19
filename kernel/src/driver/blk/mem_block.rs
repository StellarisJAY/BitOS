use simplefs::block_device::BlockDevice;
use simplefs::layout::BLOCK_SIZE;

// 内存块设备，在.data创建的内存文件系统
pub struct MemoryBlockDevice {
    start: usize,
    end: usize,
}

impl MemoryBlockDevice {
    pub fn new() -> Self {
        // see asm/link_fs.S
        extern "C" {
            fn _fs_start();
            fn _fs_end();
        }
        kernel!("using Memory block device, range: [{:#x}, {:#x}), size: {} MiB", _fs_start as usize, _fs_end as usize, 
        (_fs_end as usize - _fs_start as usize) >> 20);
        Self {
            start: _fs_start as usize,
            end: _fs_end as usize,
        }
    }

    fn block_id_to_mem_addr(&self, block_id: u32) -> usize {
        self.start + (BLOCK_SIZE * block_id) as usize
    }
}

impl BlockDevice for MemoryBlockDevice {
    fn read(&self, block_id: u32, data: &mut [u8]) {
        let offset = self.block_id_to_mem_addr(block_id);
        if offset >= self.end {
            error!("offset out of range, blk: {}, off: {:#x}, end: {:#x}", block_id, offset, self.end);
            panic!("mem blk read out of range");
        }
        unsafe {
            let ptr = offset as *const u8;
            let block = core::slice::from_raw_parts(ptr, data.len());
            data.copy_from_slice(block);
        }
    }

    fn write(&self, block_id: u32, data: &[u8]) {
        let offset = self.block_id_to_mem_addr(block_id);
        if offset >= self.end {
            error!("offset out of range, blk: {}, off: {:#x}, end: {:#x}", block_id, offset, self.end);
            panic!("mem blk write out of range");
        }
        unsafe {
            let ptr = offset as *mut u8;
            let block = core::slice::from_raw_parts_mut(ptr, data.len());
            block.copy_from_slice(data);
        }
    }
}
