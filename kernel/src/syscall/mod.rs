pub mod fs;
pub mod proc;
pub mod task;
pub mod sync;
pub mod time;

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
const SYSCALL_WAIT_TID: usize = 1001;

const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;

pub fn handle_syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_WRITE => {
            return fs::sys_write(args[0], args[1], args[2]);
        }
        SYSCALL_EXIT => proc::sys_exit(args[0] as i32),
        SYSCALL_YIELD => proc::sys_yield(),
        SYSCALL_READ => fs::sys_read(args[0], args[1], args[2]),
        SYSCALL_FORK => proc::sys_fork() as isize,
        SYSCALL_CREATE_THREAD => task::create_thread(args[0], args[1]) as isize,
        SYSCALL_WAIT_TID => task::wait_tid(args[0]),
        SYSCALL_WAITPID => proc::sys_waitpid(args[0]),
        SYSCALL_SPAWN => proc::sys_spawn(args[0], args[1]),
        SYSCALL_MUTEX_CREATE => sync::mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sync::mutex_lock(args[0] as isize),
        SYSCALL_MUTEX_UNLOCK => sync::mutex_unlock(args[0] as isize),
        SYSCALL_GET_TIME => time::syscall_get_time(args[0]),
        _ => {
            debug!("unsupported syscall: {}", id);
            panic!("unsupported syscall");
        }
    }
}
