use super::layout::BLOCK_SIZE;

#[derive(PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
}

pub const INODE_SIZE: u32 = 128;
const DIRECT_DATA_BLOCK_COUNT: u32 = 24;
const DIRECT_SIZE: u32 = DIRECT_DATA_BLOCK_COUNT * BLOCK_SIZE as u32;
const IDX_COUNT_PER_BLOCK: u32 = 1024;
// 每级索引能够独自映射的data blocks数量
const IDX1_BLOCK_COUNT: u32 = IDX_COUNT_PER_BLOCK;
const IDX2_BLOCK_COUNT: u32 = IDX_COUNT_PER_BLOCK * IDX_COUNT_PER_BLOCK;
// 每级索引能够独自映射的文件大小
const IDX1_SIZE: u32 = IDX1_BLOCK_COUNT * BLOCK_SIZE as u32;

// 一个inode块，大小128字节
#[repr(align(128))]
pub struct DiskInode{
    size: u32,                      
    direct: [u32;DIRECT_DATA_BLOCK_COUNT as usize],         // 直接映射的datanodes，24个 * 4KiB = 96KiB
    index1: u32,                                       // 一级索引，1个索引块 * 1024 * 4KiB = 4MiB
    index2: u32,                                       // 二级索引，1个索引块 * 1024 * 1024 * 4KiB = 4GiB
    inode_type: InodeType,
}

impl DiskInode {
    pub fn new(inode_type: InodeType) -> Self {
        return Self { size: 0, direct: [0; DIRECT_DATA_BLOCK_COUNT as usize], index1: 0, index2: 0, inode_type: inode_type };
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
}

fn data_blocks_for_size(size: u32) -> u32 {
    return (size + BLOCK_SIZE - 1) / BLOCK_SIZE;
}

fn index_blocks_for_size(size: u32) -> u32 {
    let mut data_blocks = data_blocks_for_size(size);
    let mut blocks: u32 = 0;
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

#[allow(unused)]
pub fn test_blocks_for_size() {
    const N: usize = 3;
    let cases: [u32; N] = [86, 3000, 4096 * 1024];
    let expect_data_blocks: [u32; N] = [1, 1, 1024];
    let expect_index_blocks: [u32; N] = [0, 0, 1];
    for (i, c) in cases.iter().enumerate() {
        let d_blocks = data_blocks_for_size(*c);
        assert!(expect_data_blocks[i] == d_blocks, "case {} failed, expect d_bloccks: {}, got: {}", i, expect_data_blocks[i], d_blocks);
        let i_blocks = index_blocks_for_size(*c);
        assert!(expect_index_blocks[i] == i_blocks, "case {} failed, expect i_bloccks: {}, got: {}", i, expect_index_blocks[i], i_blocks);
    }
}