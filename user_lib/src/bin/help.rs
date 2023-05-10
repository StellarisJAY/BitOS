#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
#[no_mangle]
pub fn main() -> i32 {
    println!("Available commands: ");
    println!("hello_world: run the Rust hello world programe");
    println!("fork_test:   run a fork and waitpid test");
    println!("thread_test: run a multi-thread test programe");
    println!("shell:       open a new shell");
    return 0;
}