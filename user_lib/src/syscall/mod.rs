use core::arch::asm;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FORK: usize = 220;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

const SYSCALL_CREATE_THREAD: usize = 1000;
const SYSCALL_WAITTID: usize = 1001;

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

pub fn read(fd: usize ,buf: &[u8]) -> isize {
    ecall(SYSCALL_READ, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn fork() -> isize {
    ecall(SYSCALL_FORK, [0usize; 3])
}

pub fn yield_() -> isize {
    ecall(SYSCALL_YIELD, [0usize;3])
}

pub fn wait_pid(pid: usize) -> isize {
    ecall(SYSCALL_WAITPID, [pid, 0, 0])
}

pub fn create_thread(entry: usize, args: usize) -> isize {
    ecall(SYSCALL_CREATE_THREAD, [entry, args, 0])
}

pub fn wait_tid(tid: isize) -> isize {
    ecall(SYSCALL_WAITTID, [tid as usize, 0, 0])
}