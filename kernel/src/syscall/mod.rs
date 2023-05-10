pub mod fs;
pub mod proc;
pub mod task;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FORK: usize = 220;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

const SYSCALL_CREATE_THREAD: usize = 1000;
const SYSCALL_WAIT_TID: usize = 1001;

pub fn handle_syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_WRITE => {
            return fs::sys_write(args[0], args[1], args[2]);
        }
        SYSCALL_EXIT => proc::sys_exit(args[0] as i32),
        SYSCALL_YIELD => proc::sys_yield(),
        SYSCALL_READ => {
            return fs::sys_read(args[0], args[1], args[2]);
        }
        SYSCALL_FORK => {
            return proc::sys_fork() as isize;
        }
        SYSCALL_CREATE_THREAD => return task::create_thread(args[0]) as isize,
        SYSCALL_WAIT_TID => return task::wait_tid(args[0]),
        SYSCALL_WAITPID => return proc::sys_waitpid(args[0]),
        _ => {
            debug!("unsupported syscall: {}", id);
            panic!("unsupported syscall");
        }
    }
}
