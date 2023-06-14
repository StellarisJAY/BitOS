use crate::config::{TIME_FREQ_MS, TIME_FREQ_US};
use crate::task::scheduler::current_task;
use crate::timer::get_time;
#[repr(C)]
struct TimerResult {
    msec: usize,
    usec: usize,
}

pub fn syscall_get_time(res: usize) -> isize {
    unsafe {
        let pa = current_task().transalate_virtaddr(res);
        let result = pa as *mut TimerResult;
        let time = get_time();
        (*result).msec = time / TIME_FREQ_MS;
        (*result).usec = time / TIME_FREQ_US;
        0
    }
}
