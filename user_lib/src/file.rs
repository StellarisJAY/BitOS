use crate::syscall;
use crate::{read, write};
use bitflags::bitflags;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
    }
}

pub enum SeekFrom {
    START,
    CUR,
    END,
}

pub struct File(usize);

// 文件状态struct
#[repr(C)]
#[derive(Debug)]
pub struct FileStat {
    pub inode: u32,        // inode编号
    pub size: u32,         // 大小
    pub blocks: u32,       // 占用的IO块总数
    pub io_block: u32,     // IO块大小
    pub index_blocks: u32, // 索引块数量
}

fn open(path: &str, flags: OpenFlags) -> isize {
    syscall::open(path, flags.bits())
}

fn close(fd: usize) -> isize {
    syscall::close(fd)
}

pub fn stat(path: &str) -> Option<FileStat> {
    let mut file_stat = FileStat::empty();
    if syscall::stat(path, &mut file_stat as *mut _ as usize) == 0 {
        return Some(file_stat);
    }
    None
}

impl File {
    pub fn open(path: &str, flags: OpenFlags) -> Option<Self> {
        let fd = open(path, flags);
        if fd == -1 {
            return None;
        } else {
            return Some(Self(fd as usize));
        }
    }

    pub fn close(&self) -> isize {
        close(self.0)
    }

    pub fn read(&self, buf: &mut [u8]) -> isize {
        read(self.0, buf)
    }

    pub fn write(&self, buf: &[u8]) -> isize {
        write(self.0, buf)
    }

    pub fn fstat(&self) -> Option<FileStat> {
        let mut file_stat = FileStat::empty();
        if syscall::fstat(self.0, &mut file_stat as *mut _ as usize) == 0 {
            return Some(file_stat);
        }
        None
    }

    pub fn lseek(&self, offset: u32, from: SeekFrom) -> isize {
        let from_val: u8 = match from {
            SeekFrom::START => 0,
            SeekFrom::CUR => 1,
            SeekFrom::END => 2,
        };
        return syscall::lseek(self.0, offset, from_val);
    }
}

impl FileStat {
    fn empty() -> Self {
        Self {
            inode: 0,
            size: 0,
            blocks: 0,
            io_block: 0,
            index_blocks: 0,
        }
    }
}
