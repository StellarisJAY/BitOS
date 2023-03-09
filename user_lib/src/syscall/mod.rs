use core::arch::asm;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_WRITE: usize = 64;

// ecall 系统调用
fn ecall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!("ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x17") id);
    }
    return ret;
}

pub fn exit(code: i32) {
    ecall(SYSCALL_EXIT, [code as usize, 0, 0]);
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    ecall(SYSCALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

