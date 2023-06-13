use crate::task::scheduler::current_proc;
use crate::fs::UserBuffer;
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
