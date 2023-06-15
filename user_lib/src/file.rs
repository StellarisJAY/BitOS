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

pub struct File(usize);

fn open(path: &str, flags: OpenFlags) -> isize {
    syscall::open(path, flags.bits())
}

fn close(fd: usize) -> isize {
    syscall::close(fd)
}

impl File {
    pub fn open(path: &str, flags: OpenFlags) -> Self {
        Self(open(path, flags) as usize)        
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
}