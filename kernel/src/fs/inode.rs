use super::{File, FileStat, UserBuffer};
use crate::driver::blk::BLOCK_DEVICE;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::borrow::Borrow;
use lazy_static::lazy_static;
use simplefs::simple_fs::SimpleFileSystem;
use simplefs::vfs::{Inode, FILE_EXIST_ERROR, FILE_NOT_FOUND_ERROR, NOT_DIR_ERROR};
use spin::mutex::Mutex;

lazy_static! {
    pub static ref ROOT_INODE: Arc<OSInode> = {
        let fs = Arc::clone(&FILE_SYSTEM);
        let root = fs.lock().root_inode(Arc::clone(&FILE_SYSTEM));
        return Arc::new(OSInode::new(true, true, Arc::new(root)));
    };
}

lazy_static! {
    pub static ref FILE_SYSTEM: Arc<Mutex<SimpleFileSystem>> = {
        let file_system = Arc::new(Mutex::new(SimpleFileSystem::open(Arc::clone(
            &BLOCK_DEVICE,
        ))));
        kernel!("file system detected and opened");
        return file_system;
    };
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const DIR = 1 << 8;
    }
}

const SEEK_SET: u8 = 0;
const SEEK_CUR: u8 = 1;
const SEEK_END: u8 = 2;

// 内核inode
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: u32,       // 读写位置offset
    inode: Arc<Inode>, // 文件系统inode
}

pub fn open_file(path: &str, flags: OpenFlags) -> Result<Arc<OSInode>, isize> {
    let (readable, writable) = flags.is_read_write();
    if flags.contains(OpenFlags::CREATE) {
        // 文件是否存在，不存在时需要创建
        if let Ok(inode) = find(path) {
            return Ok(inode);
        } else {
            return create(path, flags.is_dir(), readable, writable);
        }
    } else {
        return find(path);
    }
}

pub fn find(path: &str) -> Result<Arc<OSInode>, isize> {
    // 根目录
    if path.is_empty() || path == "/" {
        return Ok(ROOT_INODE.clone());
    }
    let s = String::from(path);
    let parts: Vec<_> = s.split("/").collect();
    let mut cur_inode = Arc::clone(&ROOT_INODE);
    let depth = parts.len();
    for (i, part) in parts.iter().enumerate() {
        if *part == "" {
            continue;
        }
        if let Some(next_inode) = cur_inode.find(*part) {
            if i != depth - 1 && !next_inode.is_dir() {
                return Err(NOT_DIR_ERROR);
            }
            cur_inode = Arc::new(next_inode);
        } else {
            return Err(FILE_NOT_FOUND_ERROR);
        }
    }
    return Ok(cur_inode);
}

fn create(path: &str, dir: bool, readable: bool, writable: bool) -> Result<Arc<OSInode>, isize> {
    let s = String::from(path);
    let mut parts: Vec<_> = s.split("/").collect();
    let filename = parts.pop().unwrap();
    let mut cur_inode = Arc::clone(&ROOT_INODE);
    let depth = parts.len();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(next_inode) = cur_inode.find(*part) {
            if !next_inode.is_dir() {
                return Err(NOT_DIR_ERROR);
            }
            cur_inode = Arc::new(next_inode);
        } else {
            return Err(FILE_NOT_FOUND_ERROR);
        }
    }
    return cur_inode.create(filename, dir, readable, writable);
}

#[allow(unused)]
pub fn list_apps() {
    let apps = ROOT_INODE.ls().unwrap();
    kernel!("listing kernel apps: ");
    apps.iter().enumerate().for_each(|(i, name)| {
        kernel!(
            "{}. {}, size: {}",
            i,
            name,
            ROOT_INODE.find(name).unwrap().size()
        );
    });
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: Mutex::new(OSInodeInner { offset: 0, inode }),
        }
    }

    pub fn ls(&self) -> Option<Vec<String>> {
        self.inner.lock().inode.ls()
    }

    pub fn find(&self, name: &str) -> Option<OSInode> {
        let inner = self.inner.lock();
        inner
            .inode
            .find(name)
            .map(|inode| OSInode::new(true, true, Arc::new(inode)))
    }

    pub fn size(&self) -> u32 {
        self.inner.lock().inode.size()
    }

    pub fn create(
        &self,
        name: &str,
        dir: bool,
        readable: bool,
        writable: bool,
    ) -> Result<Arc<OSInode>, isize> {
        let res = self.inner.lock().inode.create(name, dir);
        return res.map(|inode| Arc::new(OSInode::new(readable, writable, inode)));
    }

    pub fn read_stat(&self, stat: &mut FileStat) {
        let inner = self.inner.borrow();
        let inode_stat = inner.lock().inode.read_stat();
        stat.blocks = inode_stat.blocks;
        stat.index_blocks = inode_stat.index_blocks;
        stat.io_block = inode_stat.io_block;
        stat.size = inode_stat.size;
        stat.inode = inode_stat.inode;
        stat.dir = inode_stat.dir;
    }

    pub fn is_dir(&self) -> bool {
        self.inner.lock().inode.is_dir()
    }
}

impl File for OSInode {
    fn read<'a>(&self, buf: &mut UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let size = inner.inode.size();
        if inner.offset == size {
            return 0;
        }
        let mut read_len: usize = 0;
        buf.foreach(|bytes| {
            inner.inode.read(inner.offset, bytes);
            read_len += bytes.len().min((size - inner.offset) as usize);
            inner.offset = (inner.offset + bytes.len() as u32).min(size);
            return inner.offset == size;
        });
        return read_len;
    }
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        buf.foreach(|bytes| {
            inner.inode.write(inner.offset, bytes);
            inner.offset += bytes.len() as u32;
            return true;
        });
        return inner.offset as usize;
    }

    fn fstat(&self) -> Option<FileStat> {
        let mut stat = FileStat::empty();
        self.read_stat(&mut stat);
        Some(stat)
    }

    fn lseek(&self, off: u32, from: u8) -> isize {
        let offset: usize;
        let mut inner = self.inner.lock();
        let size = inner.inode.size();
        match from {
            SEEK_SET => inner.offset = off.min(size),
            SEEK_CUR => inner.offset = (inner.offset + off).min(size),
            SEEK_END => inner.offset = size - off,
            _ => return -1,
        }
        return inner.offset as isize;
    }
}

impl OSInode {
    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.lock();
        let mut remain = inner.inode.size();
        let mut data: Vec<u8> = Vec::new();
        let mut offset = 0u32;
        let mut buf: [u8; 512] = [0; 512];
        while remain > 0 {
            let len = inner
                .inode
                .read(offset, &mut buf[0..512.min(remain as usize)]);
            remain -= len as u32;
            offset += len as u32;
            data.extend_from_slice(&buf[0..len]);
        }
        return data;
    }
}

impl OpenFlags {
    pub fn is_read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }

    fn is_dir(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.contains(OpenFlags::DIR)
    }
}
