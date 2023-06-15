#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
#[no_mangle]
pub fn main() -> i32 {
    println!("available commands: ");
    println!("hello_world:                 run the Rust hello world");
    println!("fork_test:                   run a fork and waitpid test");
    println!("thread_test:                 run a multi-thread test");
    println!("timeshard_test:              run a test to see Round-robin with TimeShards");
    println!("shell:                       open a new shell");
    println!("exit:                        close current shell");
    return 0;
}
