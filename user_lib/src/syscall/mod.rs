use core::arch::asm;

const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_STAT: usize = 2001;
const SYSCALL_FSTAT: usize = 2002;
const SYSCALL_LSEEK: usize = 2003;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_FORK: usize = 220;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_SPAWN: usize = 1220;

const SYSCALL_CREATE_THREAD: usize = 1000;
const SYSCALL_WAITTID: usize = 1001;

const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;
const SYSCALL_COND_CREATE: usize = 1030;
const SYSCALL_COND_WAIT: usize = 1032;
const SYSCALL_COND_SIGNAL: usize = 1031;

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

pub fn read(fd: usize, buf: &[u8]) -> isize {
    ecall(SYSCALL_READ, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn fork() -> isize {
    ecall(SYSCALL_FORK, [0usize; 3])
}

pub fn yield_() -> isize {
    ecall(SYSCALL_YIELD, [0usize; 3])
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

pub fn spawn(name: &str, args: &[*const u8]) -> isize {
    ecall(
        SYSCALL_SPAWN,
        [name.as_ptr() as usize, args.as_ptr() as usize, args.len()],
    )
}

pub fn mutex_create(blocking: bool) -> isize {
    ecall(SYSCALL_MUTEX_CREATE, [if blocking { 1 } else { 0 }, 0, 0])
}

pub fn mutex_lock(id: isize) -> isize {
    ecall(SYSCALL_MUTEX_LOCK, [id as usize, 0, 0])
}

pub fn mutex_unlock(id: isize) -> isize {
    ecall(SYSCALL_MUTEX_UNLOCK, [id as usize, 0, 0])
}

pub fn get_time(res_ptr: usize) -> isize {
    ecall(SYSCALL_GET_TIME, [res_ptr, 0, 0])
}

pub fn cond_create() -> isize {
    ecall(SYSCALL_COND_CREATE, [0, 0, 0])
}

pub fn cond_signal(id: isize) -> isize {
    ecall(SYSCALL_COND_SIGNAL, [id as usize, 0, 0])
}

pub fn cond_wait(id: isize, mutex: isize) -> isize {
    ecall(SYSCALL_COND_WAIT, [id as usize, mutex as usize, 0])
}

pub fn open(path: &str, flags: u32) -> isize {
    ecall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn close(fd: usize) -> isize {
    ecall(SYSCALL_CLOSE, [fd, 0, 0])
}

pub fn stat(path: &str, stat_ptr: usize) -> isize {
    ecall(SYSCALL_STAT, [path.as_ptr() as usize, stat_ptr, 0])
}

pub fn fstat(fd: usize, stat_ptr: usize) -> isize {
    ecall(SYSCALL_FSTAT, [fd, stat_ptr, 0])
}


pub fn lseek(fd: usize, offset: u32, from: u8) -> isize {
    ecall(SYSCALL_LSEEK, [fd, offset as usize, from as usize])
}