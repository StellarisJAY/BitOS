// 一个磁盘块的大小：4KiB
pub const BLOCK_SIZE: u32 = 4096;

#[repr(C)]
pub struct SuperBlock {
    magic_number: u64,        // 文件系统识别magic num
    inode_bmap_blocks: u32, // inode bitmap块数量
    data_bmap_blocks: u32,  // data bitmap块数量
    inode_blocks: u32,      // inode块总数
    data_blocks: u32,       // 数据块总数
}

