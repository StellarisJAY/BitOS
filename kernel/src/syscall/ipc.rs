use crate::task::scheduler::current_proc;
use crate::ipc::pipe::create_pipe;

pub fn sys_pipe(addr: usize) -> isize {
    let proc = current_proc();
    let fd_table: &mut [usize];
    unsafe {
        let ptr = proc.translate_va(addr) as *mut usize;
        fd_table = core::slice::from_raw_parts_mut(ptr, 2);
    }
    let (r_fd, w_fd) = (proc.alloc_fd(), proc.alloc_fd());
    let (r_pipe, w_pipe) = create_pipe();
    
    proc.borrow_inner().fd_table[r_fd] = Some(r_pipe);
    proc.borrow_inner().fd_table[w_fd] = Some(w_pipe);
    fd_table[0] = r_fd;
    fd_table[1] = w_fd;
    0
}