use crate::fs::{File, UserBuffer};
use crate::task::scheduler::yield_current_task;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::mutex::Mutex;

const RING_BUFFER_SIZE: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PipeStatus {
    FULL,
    EMPTY,
    AVAILABLE,
}

pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

pub struct PipeRingBuffer {
    array: [u8; RING_BUFFER_SIZE],
    read_idx: usize,
    write_idx: usize,
    status: PipeStatus,
    write_end: Option<Weak<Pipe>>,
}

// 创建一个管道，返回写端和读端
pub fn create_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let write_end = Arc::new(Pipe::new(false, true, Arc::clone(&buffer)));
    let read_end = Arc::new(Pipe::new(true, false, Arc::clone(&buffer)));
    buffer.lock().set_write_end(&write_end);
    return (read_end, write_end);
}

impl PipeRingBuffer {
    fn new() -> Self {
        Self {
            array: [0u8; RING_BUFFER_SIZE],
            read_idx: 0,
            write_idx: 0,
            status: PipeStatus::EMPTY,
            write_end: None,
        }
    }

    fn set_write_end(&mut self, pipe: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(pipe));
    }

    fn available_bytes(&self) -> usize {
        if self.status == PipeStatus::EMPTY {
            return 0;
        } else {
            if self.write_idx > self.read_idx {
                return self.write_idx - self.read_idx;
            } else {
                return self.write_idx + RING_BUFFER_SIZE - self.read_idx;
            }
        }
    }

    fn write_end_closed(&self) -> bool {
        if let Some(write_end) = &self.write_end {
            return write_end.upgrade().unwrap().writable;
        } else {
            return false;
        }
    }
}

impl Pipe {
    fn new(read: bool, write: bool, buf: Arc<Mutex<PipeRingBuffer>>) -> Self {
        assert!(read != write);
        Self {
            readable: read,
            writable: write,
            buffer: buf,
        }
    }
}

// todo read write
impl File for Pipe {
    fn read<'a>(&self, buf: &mut UserBuffer) -> usize {
        panic!("not implemented");
        0
    }
    fn write<'a>(&self, buf: &mut UserBuffer) -> usize {
        panic!("not implemented");
        0
    }
    fn fstat(&self) -> Option<crate::fs::FileStat> {
        None
    }
    fn lseek(&self, offset: u32, from: u8) -> isize {
        -1
    }
}
