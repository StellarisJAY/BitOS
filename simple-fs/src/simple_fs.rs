use alloc::sync::Arc;
use crate::block_device::BlockDevice;
use crate::bitmap::{Bitmap, ALLOC_PER_BMAP_BLOCK};
use crate::inode::INODES_PER_BLOCK;
use crate::block_cache::get_block_cache_entry;
use crate::super_block::SuperBlock;
use crate::layout::BLOCK_SIZE;

pub struct SimpleFileSystem {
    pub block_dev: Arc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_start: u32,
    data_start: u32,
}

impl SimpleFileSystem {
    // 在块设备上创建一个文件系统
    pub fn new(block_dev: Arc<dyn BlockDevice>, total_blocks: u32, inodes: u32) -> Self {
        let inode_blocks = inodes * INODES_PER_BLOCK;
        let inode_bmap_blocks = inode_blocks / ALLOC_PER_BMAP_BLOCK;
        // 总块数减去一个超级块和inode块 = data块 + data_bmap块
        let remaining = total_blocks - inode_blocks - inode_bmap_blocks - 1;
        // 剩下的block里面，分成多个{一个bitmap块+可分配的data块}组合
        let data_bmap_blocks = remaining / (ALLOC_PER_BMAP_BLOCK + 1);
        let data_blocks = data_bmap_blocks * ALLOC_PER_BMAP_BLOCK;
        
        // 清空缓存数据
        for i in 0..total_blocks {
            get_block_cache_entry(i, Arc::clone(&block_dev))
            .unwrap()
            .lock()
            .modify(0, |data: &mut [u8; BLOCK_SIZE as usize]| {
                data.fill(0);
            });
        }
        
        // 写入超级块
        get_block_cache_entry(0, Arc::clone(&block_dev))
        .unwrap()
        .lock()
        .modify(0, |super_blk: &mut SuperBlock| {
            *super_blk = SuperBlock::new(inode_blocks, data_blocks);
        });
        let first_inode_bmap_blk = 1;
        let first_data_bmap_blk = first_inode_bmap_blk + inode_bmap_blocks;
        let first_inode_block = first_data_bmap_blk + data_bmap_blocks;
        let first_data_block = first_inode_block + inode_blocks;
        // 创建bitmap
        let inode_bmap = Bitmap::new(first_inode_block, first_inode_bmap_blk, inode_bmap_blocks);
        let data_bmap = Bitmap::new(first_data_block, first_data_bmap_blk, data_bmap_blocks);
        return Self { block_dev: block_dev, inode_bitmap: inode_bmap, data_bitmap: data_bmap, inode_start: first_inode_block, data_start: first_data_block };
    }
    
    // 从块设备上打开文件系统
    pub fn open(block_dev: Arc<dyn BlockDevice>) -> Self {
        let super_blk: &SuperBlock = get_block_cache_entry(0, Arc::clone(&block_dev))
        .unwrap()
        .lock()
        .as_ref(0);
        if !super_blk.verify() {
            panic!("invalid file system");
        }
        let first_inode_bmap_blk = 1;
        let first_data_bmap_blk = first_inode_bmap_blk + super_blk.inode_bmap_blocks;
        let first_inode_block = first_data_bmap_blk + super_blk.data_bmap_blocks;
        let first_data_block = first_inode_block + super_blk.inode_blocks;
        // 创建bitmap
        let inode_bmap = Bitmap::new(first_inode_block, first_inode_bmap_blk, super_blk.inode_bmap_blocks);
        let data_bmap = Bitmap::new(first_data_block, first_data_bmap_blk, super_blk.data_bmap_blocks);
        return Self { block_dev: block_dev, inode_bitmap: inode_bmap, data_bitmap: data_bmap, inode_start: first_inode_block, data_start: first_data_block };
    }
}