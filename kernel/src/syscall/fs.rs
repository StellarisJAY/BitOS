use crate::task::scheduler::current_proc;
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let proc = current_proc();
    let buf = proc.translate_buffer(buf_ptr, len);
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.write(buf) as isize;
    }
    0
}

pub fn sys_read(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let proc = current_proc();
    let buf = proc.translate_buffer(buf_ptr, len);
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.read(buf) as isize;
    }
    0
}
