use super::block_cache::get_block_cache_entry;
use super::block_device::BlockDevice;
use super::layout::BLOCK_SIZE;
use alloc::sync::Arc;

const BITMAP_SIZE: usize = BLOCK_SIZE / 8;
const ALLOC_PER_BMAP_BLOCK: usize = BLOCK_SIZE * 8;

// BitmapBlock 一个bitmap块，其中每个二进制位表示一个被管理块是否可分配
// 一个bmap块=4KiB，为512个u64，总共512 * 64 = 32K个id
// 32k个id，每个id为4KiB磁盘块，所以一个bimap块可以管理128MiB空间
#[repr(C)]
struct BitmapBlock {
    bits: [u64; BITMAP_SIZE], 
}

pub struct Bitmap {
    first_block_id: usize,  // bitmap管理的区域的第一个块id
    first_bm_block: usize,  // bitmap的第一个bm块id
    total_bm_blocks: usize, // bitmap所拥有的bm块总数
}

impl Bitmap {
    pub fn new(first_block_id: usize, first_bm_block: usize, total_bm_blocks: usize) -> Self {
        return Self {
            first_block_id,
            first_bm_block,
            total_bm_blocks,
        };
    }

    pub fn alloc(&mut self, block_device: Arc<dyn BlockDevice>) -> Option<usize> {
        for seq in 0..self.total_bm_blocks {
            // 读取bmap块的数据，转换成BitMapBlock类型
            let bmap_block_id = self.first_bm_block + seq;
            let cache_entry = get_block_cache_entry(bmap_block_id, Arc::clone(&block_device)).unwrap();
            let bm_block: &mut BitmapBlock = cache_entry.lock().as_mut(0);
            // 分配块id
            let result = bm_block
                .bits
                .iter_mut()
                .enumerate()
                .find(|(_, bit)| **bit != u64::MAX) // 非全1
                .map(|(idx, bit)| {
                    let offset = bit.trailing_ones(); // 找到第一个0
                    *bit |= 1u64 << offset;           // 将0设置为1
                    return (idx, offset as usize);    // 返回第idx个u64的offset位置
                });
            if let Some((idx, offset)) = result {
                return Some(self.compose_block_id(seq, idx, offset));
            }
        }
        None
    }
    
    pub fn dealloc(&mut self, block_id: usize, block_device: Arc<dyn BlockDevice>) {
        // 获取block_id所在的bmap_block序号，block内的idx 和 u64内的offset
        let (bmap_seq, idx, offset) = self.decompose_block_id(block_id);
        // 获取block cache
        let bmap_block_id = bmap_seq + self.first_block_id;
        let cache_entry = get_block_cache_entry(bmap_block_id, Arc::clone(&block_device)).unwrap();
        // 将cache的bitmap位设置0，回收块id
        let bm_block: &mut BitmapBlock = cache_entry.lock().as_mut(0);
        let b = &mut bm_block.bits[idx];
        (*b) &= !(1<<offset);
        drop(bm_block);
    }

    // 从bmap序号，bmap块内序号，和u64的offset 获取最终的block_id
    fn compose_block_id(&self, bmap_seq: usize, idx: usize, offset: usize) -> usize {
        self.first_block_id + bmap_seq * ALLOC_PER_BMAP_BLOCK + idx * 64 + offset
    }
    
    fn decompose_block_id(&self, block_id: usize) -> (usize, usize, usize) {
        let mut id = block_id - self.first_block_id;
        let offset = id % 64;
        id /= 64;
        let idx = id % BITMAP_SIZE;
        let bmap_seq = id / BITMAP_SIZE;
        return (bmap_seq, idx, offset);
    }
}

#[allow(unused)]
pub fn test_compose_and_decompose() {
    let bmap = Bitmap::new(0, 0, 0);
    let cases = vec![(1, 1, 16), (2, 2, 12)];
    for (i,c) in cases.iter().enumerate(){
        let id = bmap.compose_block_id(c.0, c.1, c.2);
        let res = bmap.decompose_block_id(id);
        assert!((*c) == res, "case {} failed", i);
    }
}
