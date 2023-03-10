pub mod fs;
pub mod proc;

const SYSCALL_EXIT: usize = 93;
const SYSCALL_WRITE: usize = 64;

pub fn handle_syscall(id: usize, args: [usize;3]) -> isize {
    match id {
        SYSCALL_WRITE => {
            return fs::sys_write(args[0], args[1], args[2]);
        },
        SYSCALL_EXIT => {
            proc::sys_exit(args[0] as i32)
        },
        _ => panic!("unsupported syscall")
    }
}