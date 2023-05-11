use crate::bitmap::ALLOC_PER_BMAP_BLOCK;
pub const SIMPLE_FS_MAGIC: u64 = 0x73696d706c656673;

#[repr(C)]
pub struct SuperBlock {
    magic_number: u64,          // 文件系统识别magic num
    pub inode_bmap_blocks: u32, // inode bitmap块数量
    pub data_bmap_blocks: u32,  // data bitmap块数量
    pub inode_blocks: u32,      // inode块总数
    pub data_blocks: u32,       // 数据块总数
}

impl SuperBlock {
    pub fn new(inode_blocks: u32, data_blocks: u32) -> Self {
        let inode_bmap_blocks = (inode_blocks + ALLOC_PER_BMAP_BLOCK - 1) / ALLOC_PER_BMAP_BLOCK;
        let data_bmap_blocks = (data_blocks + ALLOC_PER_BMAP_BLOCK - 1) / ALLOC_PER_BMAP_BLOCK;
        return Self {
            magic_number: SIMPLE_FS_MAGIC,
            inode_bmap_blocks,
            data_bmap_blocks,
            inode_blocks,
            data_blocks,
        };
    }
    // 验证文件系统
    pub fn verify(&self) -> bool {
        return self.magic_number == SIMPLE_FS_MAGIC;
    }
}
