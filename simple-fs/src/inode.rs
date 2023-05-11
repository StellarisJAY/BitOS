use super::block_cache::get_block_cache_entry;
use super::block_cache::CacheEntry;
use super::block_device::BlockDevice;
use super::layout::BLOCK_SIZE;
use alloc::sync::Arc;
use spin::mutex::Mutex;

#[derive(PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
}

pub const INODE_SIZE: u32 = 128;
pub const INODES_PER_BLOCK: u32 = BLOCK_SIZE / INODE_SIZE;
const DIRECT_DATA_BLOCK_COUNT: u32 = 24;
const IDX_COUNT_PER_BLOCK: u32 = 1024;
// 每级索引能够独自映射的data blocks数量
const IDX1_BLOCK_COUNT: u32 = IDX_COUNT_PER_BLOCK;
const IDX2_BLOCK_COUNT: u32 = IDX_COUNT_PER_BLOCK * IDX_COUNT_PER_BLOCK;
const MAX_DATA_BLOCKS: u32 = IDX1_BLOCK_COUNT + IDX2_BLOCK_COUNT + DIRECT_DATA_BLOCK_COUNT;

// 一个inode块，大小128字节
#[repr(align(128))]
pub struct DiskInode {
    size: u32,
    direct: [u32; DIRECT_DATA_BLOCK_COUNT as usize], // 直接映射的datanodes，24个 * 4KiB = 96KiB
    index1: u32,                                     // 一级索引，1个索引块 * 1024 * 4KiB = 4MiB
    index2: u32, // 二级索引，1个索引块 * 1024 * 1024 * 4KiB = 4GiB
    inode_type: InodeType,
}

impl DiskInode {
    pub fn new(inode_type: InodeType) -> Self {
        return Self {
            size: 0,
            direct: [0; DIRECT_DATA_BLOCK_COUNT as usize],
            index1: 0,
            index2: 0,
            inode_type: inode_type,
        };
    }
    pub fn data_blocks(&self) -> u32 {
        return data_blocks_for_size(self.size);
    }
    pub fn is_dir(&self) -> bool {
        return self.inode_type == InodeType::Directory;
    }
    pub fn index_blocks(&self) -> u32 {
        return index_blocks_for_size(self.size);
    }
    pub fn total_blocks(&self) -> u32 {
        return self.data_blocks() + self.index_blocks();
    }
    pub fn size(&self) -> u32 {
        return self.size;
    }
    pub fn set_type(&mut self, f_type: InodeType) {
        self.inode_type = f_type;
    }
    // 获取文件的offset位置所属的数据块缓存
    pub fn get_block(
        &self,
        offset: u32,
        block_device: Arc<dyn BlockDevice>,
    ) -> Option<Arc<Mutex<CacheEntry>>> {
        let block_seq = offset / BLOCK_SIZE;
        let block_id = self.get_block_id(block_seq, Arc::clone(&block_device));
        return get_block_cache_entry(block_id, Arc::clone(&block_device));
    }

    // 通过文件内的块序号seq，获得块的全局ID
    pub fn get_block_id(&self, seq: u32, block_device: Arc<dyn BlockDevice>) -> u32 {
        // 检查块是否越界
        let mut blocks = seq + 1;
        assert!(blocks <= MAX_DATA_BLOCKS, "block seq out of range");
        // 直接索引的data块
        if blocks <= DIRECT_DATA_BLOCK_COUNT {
            return self.direct[blocks as usize - 1];
        }
        blocks -= DIRECT_DATA_BLOCK_COUNT;
        if blocks <= IDX1_BLOCK_COUNT {
            // 将一级索引块转换成[u32]，并获取blocks序号对应的id
            return get_block_cache_entry(self.index1, Arc::clone(&block_device))
                .unwrap()
                .lock()
                .read(0, |ids: &[u32; IDX_COUNT_PER_BLOCK as usize]| {
                    ids[blocks as usize - 1]
                });
        }
        blocks -= IDX1_BLOCK_COUNT;
        // 从二级索引块找到对应的一级索引块id
        let l1_block = get_block_cache_entry(self.index2, Arc::clone(&block_device))
            .unwrap()
            .lock()
            .read(0, |l2: &[u32; IDX_COUNT_PER_BLOCK as usize]| {
                l2[(blocks as usize - 1) / IDX_COUNT_PER_BLOCK as usize]
            });
        // 从一级索引读取块序号对应的id
        return get_block_cache_entry(l1_block, Arc::clone(&block_device))
            .unwrap()
            .lock()
            .read(0, |l1: &[u32; IDX_COUNT_PER_BLOCK as usize]| {
                l1[blocks as usize - 1]
            });
    }

    // 文件扩容，需要调用者提供新分配的数据块和索引块
    pub fn grow(
        &mut self,
        size: u32,
        blocks: Vec<u32>,
        mut index_blocks: Vec<u32>,
        block_device: Arc<dyn BlockDevice>,
    ) {
        let mut current_seq = self.data_blocks();
        self.size = size;
        for b in blocks {
            // 直接索引
            if current_seq < DIRECT_DATA_BLOCK_COUNT {
                self.direct[current_seq as usize] = b;
            } else if current_seq < IDX1_BLOCK_COUNT {
                // 一级索引内的偏移
                let offset1 = (current_seq - DIRECT_DATA_BLOCK_COUNT) as usize;
                get_block_cache_entry(self.index1, Arc::clone(&block_device))
                    .unwrap()
                    .lock()
                    .modify(0, |idx1: &mut [u32; IDX_COUNT_PER_BLOCK as usize]| {
                        idx1[offset1] = b;
                    });
            } else {
                // 从零开始的二级序号，二级索引内的偏移，二级索引对应的一级索引的偏移
                let idx2_seq = current_seq - DIRECT_DATA_BLOCK_COUNT - IDX1_BLOCK_COUNT;
                let offset2 = (idx2_seq / IDX_COUNT_PER_BLOCK) as usize;
                let offset1 = (idx2_seq % IDX_COUNT_PER_BLOCK) as usize;
                // 获取二级索引内的一级索引块ID
                let idx1_block_id = get_block_cache_entry(self.index2, Arc::clone(&block_device))
                    .unwrap()
                    .lock()
                    .modify(0, |idx2: &mut [u32; IDX_COUNT_PER_BLOCK as usize]| {
                        let mut id = idx2[offset2];
                        // 一级索引块不存在，需要分配新的块
                        if idx2[offset2] == 0 {
                            id = index_blocks.pop().unwrap();
                            idx2[offset2] = id;
                        }
                        return id;
                    });
                // 修改一级索引内对应偏移的块id
                get_block_cache_entry(idx1_block_id, Arc::clone(&block_device))
                    .unwrap()
                    .lock()
                    .modify(0, |idx1: &mut [u32; IDX_COUNT_PER_BLOCK as usize]| {
                        idx1[offset1] = b;
                    });
            }
            current_seq += 1;
        }
    }
    // 从inode索引的数据块里面读取从offset开始的size大小数据
    pub fn read(&self, offset: u32, size: u32, buf: &mut [u8], block_device: Arc<dyn BlockDevice>) {
        if offset >= self.size || offset + size > self.size || buf.len() < size as usize {
            panic!("read out of range");
        }
        let mut cur_block_seq = offset / BLOCK_SIZE;
        let mut cur_block_off = offset % BLOCK_SIZE;
        let mut cur_block_end = BLOCK_SIZE; // 非最后一个块的结束位置是BLOCK_SIZE
        let last_block_seq = (offset + size) / BLOCK_SIZE;
        let last_block_off = (offset + size) % BLOCK_SIZE;
        let mut buf_off = 0;
        while cur_block_seq <= last_block_seq {
            // 最后一个块的块内的结束位置
            if cur_block_seq == last_block_seq {
                cur_block_end = last_block_off;
            }
            // 获取当前块序号对应的数据块缓存
            let block_id = self.get_block_id(cur_block_seq, Arc::clone(&block_device));
            get_block_cache_entry(block_id, Arc::clone(&block_device))
                .unwrap()
                .lock()
                .read(0, |block: &[u8; BLOCK_SIZE as usize]| {
                    // 拷贝数据到buf中
                    let range = &block[cur_block_off as usize..cur_block_end as usize];
                    buf[buf_off..buf_off + range.len()].copy_from_slice(range);
                    buf_off += range.len();
                });
            cur_block_seq += 1;
            cur_block_off = 0;
        }
    }

    // 向inode索引的数据块的offset位置写入大小为size的数据
    pub fn write(&self, offset: u32, size: u32, buf: &[u8], block_device: Arc<dyn BlockDevice>) {
        if offset >= self.size || offset + size > self.size || buf.len() < size as usize {
            panic!("read out of range");
        }
        let mut cur_block_seq = offset / BLOCK_SIZE;
        let mut cur_block_off = offset % BLOCK_SIZE;
        let mut cur_block_end = BLOCK_SIZE; // 非最后一个块的结束位置是BLOCK_SIZE
        let last_block_seq = (offset + size) / BLOCK_SIZE;
        let last_block_off = (offset + size) % BLOCK_SIZE;
        let mut buf_off: usize = 0;
        while cur_block_seq <= last_block_seq {
            // 最后一个块的块内的结束位置
            if cur_block_seq == last_block_seq {
                cur_block_end = last_block_off;
            }
            // 获取块序号对应的数据块缓存
            let block_id = self.get_block_id(cur_block_seq, Arc::clone(&block_device));
            get_block_cache_entry(block_id, Arc::clone(&block_device))
                .unwrap()
                .lock()
                .modify(0, |block: &mut [u8; BLOCK_SIZE as usize]| {
                    // 将buf中的数据拷贝到块缓存中
                    let length = (cur_block_end - cur_block_off) as usize;
                    block[cur_block_off as usize..cur_block_end as usize]
                        .copy_from_slice(&buf[buf_off..buf_off + length]);
                    buf_off += length;
                });
            cur_block_seq += 1;
            cur_block_off = 0;
        }
    }
}

pub fn data_blocks_for_size(size: u32) -> u32 {
    return (size + BLOCK_SIZE - 1) / BLOCK_SIZE;
}

pub fn index_blocks_for_size(size: u32) -> u32 {
    let mut data_blocks = data_blocks_for_size(size);
    let mut blocks: u32 = 1;
    if data_blocks < DIRECT_DATA_BLOCK_COUNT {
        return blocks;
    }
    data_blocks -= DIRECT_DATA_BLOCK_COUNT;
    // 增加一个一级索引块
    blocks += 1;
    if data_blocks < IDX1_BLOCK_COUNT {
        return blocks;
    }
    data_blocks -= IDX1_BLOCK_COUNT;
    // 一个二级索引块
    blocks += 1;
    // 二级索引块指向的一级索引块
    blocks += data_blocks / IDX_COUNT_PER_BLOCK;
    if data_blocks % IDX_COUNT_PER_BLOCK != 0 {
        blocks += 1;
    }
    return blocks;
}

#[cfg(test)]
mod inode_tests {
    use super::*;

    #[test]
    fn test_blocks_for_size() {
        const N: usize = 3;
        let cases: [u32; N] = [86, 3000, 4096 * 1024];
        let expect_data_blocks: [u32; N] = [1, 1, 1024];
        let expect_index_blocks: [u32; N] = [1, 1, 2];
        for (i, c) in cases.iter().enumerate() {
            let d_blocks = data_blocks_for_size(*c);
            assert!(
                expect_data_blocks[i] == d_blocks,
                "case {} failed, expect d_bloccks: {}, got: {}",
                i,
                expect_data_blocks[i],
                d_blocks
            );
            let i_blocks = index_blocks_for_size(*c);
            assert!(
                expect_index_blocks[i] == i_blocks,
                "case {} failed, expect i_bloccks: {}, got: {}",
                i,
                expect_index_blocks[i],
                i_blocks
            );
        }
    }

    #[test]
    fn test_block_id_from_offset() {
        let node = DiskInode::new(InodeType::File);
    }
}
