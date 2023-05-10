#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{fork, wait_pid, exit};
#[no_mangle]
pub fn main() -> i32 {
    println!("[parent] fork test begin");
    let p1 = fork();
    if p1 == 0 {
        println!("[child] i am child");
        exit(-1);
    }else {
        let exit_code = wait_pid(p1 as usize);
        println!("[parent] p1 exit code: {}", exit_code);
    }
    return 0;
}