use crate::fs::UserBuffer;
use crate::task::scheduler::{current_proc, current_task_translate_string};
use crate::fs::inode::{open_file, OpenFlags};
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let mut buf = UserBuffer::from_current_proc(buf_ptr, len);
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.write(&mut buf) as isize;
    }
    0
}

pub fn sys_read(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let mut buf = UserBuffer::from_current_proc(buf_ptr, len);
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.read(&mut buf) as isize;
    }
    0
}

pub fn sys_open(path: usize, flags: u32) -> isize {
    let proc = current_proc();
    let name = proc.translate_string(path);
    if let Some(file) = open_file(name.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let fd = proc.alloc_fd();
        proc.borrow_inner().fd_table[fd] = Some(file);
        return fd as isize;
    }else {
        return -1;
    }
}

pub fn sys_close(fd: usize) -> isize {
    let proc = current_proc();
    let mut inner = proc.borrow_inner();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}