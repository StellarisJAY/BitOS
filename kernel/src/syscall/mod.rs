pub mod fs;
pub mod proc;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FORK: usize = 220;

pub fn handle_syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_WRITE => {
            return fs::sys_write(args[0], args[1], args[2]);
        }
        SYSCALL_EXIT => proc::sys_exit(args[0] as i32),
        SYSCALL_READ => {
            return fs::sys_read(args[0], args[1], args[2]);
        }
        SYSCALL_FORK => {
            return proc::sys_fork() as isize;
        }
        _ => panic!("unsupported syscall"),
    }
}
