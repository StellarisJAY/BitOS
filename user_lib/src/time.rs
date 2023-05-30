use crate::syscall::get_time;

#[repr(C)]
struct TimerResult {
    msec: usize,
    usec: usize,
}

impl TimerResult {
    fn empty() -> Self {
        Self{msec: 0, usec: 0}
    }
}

pub fn get_time_ms() -> usize {
    let mut res = TimerResult::empty();
    get_time(&mut res as *mut TimerResult as usize);
    return res.msec;
}

pub fn get_time_us() -> usize {
    let mut res = TimerResult::empty();
    get_time(&mut res as *mut TimerResult as usize);
    return res.usec;
}