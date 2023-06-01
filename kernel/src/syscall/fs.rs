use crate::task::scheduler::current_proc;


pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let buf = inner_pcb.memory_set.translate_buffer(buf_ptr, len); // buf_ptr是进程地址空间的地址，需要转换成kernel地址空间的buf
        return fd.write(buf) as isize;
    }
    0
}

pub fn sys_read(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let buf = inner_pcb.memory_set.translate_buffer(buf_ptr, len);
        return fd.read(buf) as isize;
    }
    0
}
