use crate::proc::scheduler::{exit_current_proc, schedule_idle, push_process, current_proc};

pub fn sys_exit(exit_code: i32) -> isize{
    exit_current_proc(exit_code);
    schedule_idle();
    return 0;
}

pub fn sys_yield() -> isize {
    let proc = current_proc();
    push_process(proc);
    schedule_idle();
    return 0;
}