use crate::proc::scheduler::current_proc_translate_buffer;
use crate::console::print_buf;
use crate::driver::uart::get_char;

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

pub fn sys_read(fd: usize, buf_ptr: usize, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert!(len == 1, "currently only supports one byte per read");
            let mut buf = current_proc_translate_buffer(buf_ptr, len);
            loop {
                if let Some(byte) = get_char() {
                    unsafe {
                        buf[0].as_mut_ptr().write_volatile(byte);
                        return 1;
                    }
                }
            }
        },
        _ => panic!("unsupported fd for read")
    }
    0
}