#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::time::get_time_ms;
use user_lib::{exit, fork, wait_pid};

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time_ms();
    println!("[parent] fork test begin");
    let p1 = fork();
    if p1 == 0 {
        println!("[child] i am child");
        exit(-1);
    } else {
        let exit_code = wait_pid(p1 as usize);
        println!(
            "[parent] p1 exit code: {}, time used: {} ms",
            exit_code,
            get_time_ms() - start
        );
    }
    return 0;
}
