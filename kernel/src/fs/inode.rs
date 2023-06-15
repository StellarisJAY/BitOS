use super::{File, UserBuffer};
use crate::driver::blk::BLOCK_DEVICE;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use simplefs::simple_fs::SimpleFileSystem;
use simplefs::vfs::Inode;
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
            let inner = inode.inner.lock();
            let inode = Arc::clone(&inner.inode);
            return Some(Arc::new(OSInode::new(readable, writable, inode)));
        } else {
            return ROOT_INODE
                .create(name, false, readable, writable)
                .map(|inode| Arc::new(inode));
        }
    } else {
        if let Some(inode) = ROOT_INODE.find(name) {
            let inner = inode.inner.lock();
            let inode = Arc::clone(&inner.inode);
            return Some(Arc::new(OSInode::new(readable, writable, inode)));
        } else {
            None
        }
    }
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

    pub fn create(&self, name: &str, dir: bool, readable: bool, writable: bool) -> Option<OSInode> {
        self.inner
            .lock()
            .inode
            .create(name, dir)
            .map(|inode| OSInode::new(readable, writable, inode))
    }
}

impl File for OSInode {
    fn read<'a>(&self, buf: &mut UserBuffer) -> usize {
        let inner = self.inner.lock();
        let mut offset: usize = 0;
        buf.foreach(|bytes| {
            inner.inode.read(offset as u32, bytes);
            offset += bytes.len();
        });
        return offset;
    }
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize {
        let inner = self.inner.lock();
        let mut offset: usize = 0;
        buf.foreach(|bytes| {
            inner.inode.write(offset as u32, bytes);
            offset += bytes.len();
        });
        return offset;
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
}
