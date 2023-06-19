use crate::block_cache::get_block_cache_entry;
use crate::block_device::BlockDevice;
use crate::inode::{data_blocks_for_size, index_blocks_for_size, DiskInode, InodeType};
use crate::layout::BLOCK_SIZE;
use crate::simple_fs::SimpleFileSystem;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

pub struct Inode {
    pub block_id: u32,
    pub offset: u32,
    fs: Arc<Mutex<SimpleFileSystem>>,
    block_dev: Arc<dyn BlockDevice>,
}

const DIR_NAME_LIMIT: usize = 27;
const DIR_ENTRY_SIZE: u32 = 32;

#[repr(C)]
pub struct DirEntry {
    name: [u8; DIR_NAME_LIMIT + 1],
    inode: u32,
}

#[repr(C)]
pub struct InodeStat {
    pub inode: u32,          // inode编号
    pub size: u32,           // 大小
    pub blocks: u32,         // 占用的IO块总数
    pub io_block: u32,       // IO块大小
    pub index_blocks: u32,   // 索引块数量
    pub dir: bool,
}

impl DirEntry {
    pub fn new(name: &str, inode: u32) -> Self {
        let mut entry = Self {
            name: [0u8; DIR_NAME_LIMIT + 1],
            inode: inode,
        };
        entry.name[0..name.len()].copy_from_slice(name.as_bytes());
        return entry;
    }

    pub fn empty() -> Self {
        Self {
            name: [0u8; DIR_NAME_LIMIT + 1],
            inode: 0,
        }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            let addr = self as *mut Self as usize;
            let ptr = addr as usize as *mut u8;
            return core::slice::from_raw_parts_mut(ptr, DIR_ENTRY_SIZE as usize);
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let addr = self as *const Self as usize;
            let ptr = addr as usize as *const u8;
            return core::slice::from_raw_parts(ptr, DIR_ENTRY_SIZE as usize);
        }
    }

    pub fn name(&self) -> &str {
        let length = self
            .name
            .iter()
            .enumerate()
            .find(|(_, b)| **b == 0)
            .map(|(idx, _)| idx)
            .unwrap();
        return core::str::from_utf8(&self.name[0..length]).unwrap();
    }
}

impl Inode {
    pub fn new(
        block_id: u32,
        offset: u32,
        fs: Arc<Mutex<SimpleFileSystem>>,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id,
            offset,
            fs,
            block_dev,
        }
    }

    pub fn from_inode_seq(
        seq: u32,
        fs: Arc<Mutex<SimpleFileSystem>>,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Self {
        let (block_id, _, offset) = fs.lock().get_inode_position(seq);
        return Self {
            block_id: block_id,
            offset: offset,
            fs: fs,
            block_dev: block_dev,
        };
    }

    pub fn read_stat(&self) -> InodeStat{
        let mut stat = self.read_disk_inode(|disk_inode| {
            InodeStat {
                size: disk_inode.size(),
                index_blocks: disk_inode.index_blocks(),
                blocks: disk_inode.total_blocks(),
                io_block: BLOCK_SIZE,
                inode: self.block_id,
                dir: disk_inode.is_dir(),
            }
        });
        stat.inode = self.fs.lock().get_inode_seq(self.block_id, self.offset);
        return stat;
    }

    fn read_disk_inode<F: FnMut(&DiskInode) -> V, V: Sized>(&self, mut f: F) -> V {
        return get_block_cache_entry(self.block_id, Arc::clone(&self.block_dev))
            .unwrap()
            .lock()
            .read(self.offset, |disk_inode: &DiskInode| f(disk_inode));
    }

    fn modify_disk_inode<F: FnMut(&mut DiskInode) -> V, V: Sized>(&self, mut f: F) -> V {
        return get_block_cache_entry(self.block_id, Arc::clone(&self.block_dev))
            .unwrap()
            .lock()
            .modify(self.offset, |disk_inode: &mut DiskInode| f(disk_inode));
    }

    pub fn find(&self, name: &str) -> Option<Inode> {
        return self
            .read_disk_inode(|disk_inode| {
                Self::find_inode(disk_inode, name, Arc::clone(&self.block_dev))
            })
            .map(|inode_seq| {
                let (block_id, _, offset) = self.fs.lock().get_inode_position(inode_seq);
                return Inode::new(
                    block_id,
                    offset,
                    Arc::clone(&self.fs),
                    Arc::clone(&self.block_dev),
                );
            });
    }

    pub fn ls(&self) -> Option<Vec<String>> {
        return self
            .read_disk_inode(|disk_inode| Self::list(disk_inode, Arc::clone(&self.block_dev)));
    }

    pub fn is_dir(&self) -> bool {
        return self.read_disk_inode(|disk_inode| disk_inode.is_dir());
    }

    fn find_inode(
        disk_inode: &DiskInode,
        name: &str,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Option<u32> {
        if let Some(files) = Self::list(disk_inode, Arc::clone(&block_dev)) {
            return files
                .iter()
                .enumerate()
                .find(|(_, filename)| String::from(name) == **filename)
                .map(|(idx, _)| {
                    let mut dir_entry = DirEntry::empty();
                    disk_inode.read(
                        idx as u32 * DIR_ENTRY_SIZE,
                        DIR_ENTRY_SIZE,
                        dir_entry.as_mut_bytes(),
                        Arc::clone(&block_dev),
                    );
                    return dir_entry.inode;
                });
        } else {
            return None;
        }
    }

    pub fn size(&self) -> u32 {
        self.read_disk_inode(|disk_inode| disk_inode.size())
    }

    fn list(disk_inode: &DiskInode, block_dev: Arc<dyn BlockDevice>) -> Option<Vec<String>> {
        let mut res: Vec<String> = Vec::new();
        if !disk_inode.is_dir() {
            return None;
        }
        let file_count = disk_inode.size() / DIR_ENTRY_SIZE;
        for i in 0..file_count {
            let mut dir_entry = DirEntry::empty();
            disk_inode.read(
                i * DIR_ENTRY_SIZE,
                DIR_ENTRY_SIZE,
                dir_entry.as_mut_bytes(),
                Arc::clone(&block_dev),
            );
            res.push(String::from(dir_entry.name()));
        }
        return Some(res);
    }

    pub fn create(&self, name: &str, mkdir: bool) -> Option<Arc<Inode>> {
        // 修改当前inode对应的disk inode，返回是否是dir，文件是否已经存在，以及文件的inode号
        let (is_dir, file_exists, inode_seq) = self.modify_disk_inode(|disk_inode| {
            if !disk_inode.is_dir() {
                return (false, false, None);
            }
            if let Some(_) = Self::find_inode(disk_inode, name, Arc::clone(&self.block_dev)) {
                return (true, true, None);
            }
            let mut file_system = self.fs.lock();
            let inode_seq = file_system.alloc_inode().unwrap();
            drop(file_system);
            let offset = disk_inode.size();
            // 当前inode扩容
            self.grow_disk_inode(disk_inode, disk_inode.size() + DIR_ENTRY_SIZE);
            // 写入新文件的dir条目
            let entry = DirEntry::new(name, inode_seq);
            disk_inode.write(
                offset,
                DIR_ENTRY_SIZE,
                entry.as_bytes(),
                Arc::clone(&self.block_dev),
            );
            return (true, false, Some(inode_seq));
        });
        if !is_dir || file_exists {
            return None;
        }
        // 创建inode和diskinode
        let (block_id, _, offset) = self.fs.lock().get_inode_position(inode_seq.unwrap());
        let inode = Inode::new(
            block_id,
            offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_dev),
        );
        // 设置disk inode的类型
        inode.modify_disk_inode(|disk_inode| {
            if mkdir {
                disk_inode.set_type(InodeType::Directory);
            } else {
                disk_inode.set_type(InodeType::Directory);
            }
        });
        return Some(Arc::new(inode));
    }

    pub fn read(&self, offset: u32, buf: &mut [u8]) -> usize {
        return self.read_disk_inode(|disk_inode| {
            return disk_inode.read(offset, buf.len() as u32, buf, Arc::clone(&self.block_dev));
        });
    }

    pub fn write(&self, offset: u32, buf: &[u8]) -> usize {
        return self.modify_disk_inode(|disk_inode| {
            self.grow_disk_inode(disk_inode, disk_inode.size() + buf.len() as u32);
            return disk_inode.write(offset, buf.len() as u32, buf, Arc::clone(&self.block_dev));
        });
    }

    // 扩容disk inode到目标大小
    fn grow_disk_inode(&self, disk_inode: &mut DiskInode, size: u32) {
        let old_idx_blks = disk_inode.index_blocks();
        let new_idx_blks = index_blocks_for_size(size);
        let old_data_blks = disk_inode.data_blocks();
        let new_data_blks = data_blocks_for_size(size);
        // 为disk inode分配数据块和一二级索引块
        let mut idx_blks: Vec<u32> = Vec::new();
        let mut data_blks: Vec<u32> = Vec::new();
        let mut fs = self.fs.lock();
        for _ in old_idx_blks..new_idx_blks {
            idx_blks.push(fs.alloc_data_block().unwrap());
        }
        for _ in old_data_blks..new_data_blks {
            data_blks.push(fs.alloc_data_block().unwrap());
        }
        disk_inode.grow(size, data_blks, idx_blks, Arc::clone(&self.block_dev));
    }
}
