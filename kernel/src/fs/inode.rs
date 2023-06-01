use super::File;
use crate::driver::blk::BLOCK_DEVICE;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use simplefs::simple_fs::SimpleFileSystem;
use simplefs::vfs::Inode;
use spin::mutex::Mutex;

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let fs = Arc::clone(&FILE_SYSTEM);
        let root = fs.lock().root_inode(Arc::clone(&FILE_SYSTEM));
        return Arc::new(root);
    };
}

lazy_static! {
    pub static ref FILE_SYSTEM: Arc<Mutex<SimpleFileSystem>> = {
        let file_system = Arc::new(Mutex::new(SimpleFileSystem::open(Arc::clone(
            &BLOCK_DEVICE,
        ))));
        return file_system;
    };
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
    }
}

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

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.is_read_write();
    if flags.contains(OpenFlags::CREATE) {
        // 文件是否存在，不存在时需要创建
        if let Some(inode) = ROOT_INODE.find(name) {
            let inode = Arc::new(inode);
            return Some(Arc::new(OSInode::new(readable, writable, inode)));
        } else {
            return ROOT_INODE
                .create(name, false)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)));
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            let inode = Arc::new(inode);
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

pub fn list_apps() {
    let apps = ROOT_INODE.ls().unwrap();
    apps.iter().enumerate().for_each(|(i, name)| {
        kernel!("app {}: {}", i, name);
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
}

impl File for OSInode {
    fn read<'a>(&self, mut buf: Vec<&'a mut [u8]>) -> usize {
        let inner = self.inner.lock();
        let mut offset: usize = 0;
        for slice in buf.iter_mut() {
            inner.inode.read(offset as u32, *slice);
            offset += (*slice).len();
        }
        return offset;
    }
    fn write<'a>(&self, buf: Vec<&'a mut [u8]>) -> usize {
        let inner = self.inner.lock();
        let mut offset: usize = 0;
        for slice in buf.iter() {
            inner.inode.write(offset as u32, *slice);
            offset += (*slice).len();
        }
        return offset;
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
}
