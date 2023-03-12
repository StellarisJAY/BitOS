use crate::proc::scheduler::current_proc_translate_buffer;
use crate::console::print_buf;
const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = current_proc_translate_buffer(buf_ptr, len);
            for buf in buffers {
                print_buf(buf);
            }
            return len as isize;
        },
        _ => panic!("unsupported fd for write")
    }
}