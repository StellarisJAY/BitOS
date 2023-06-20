#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
#[no_mangle]
pub fn main() -> i32 {
    println!("Shell builtin commands: ");
    println!("type                         Check the type of a command");
    println!("cd                           Change the shell working directory.");
    println!("exit                         Exit current shell process");
    print!("\n");
    println!("Applications: ");
    println!("hello_world                  Run the Rust hello world");
    println!("cat                          Print file's content");
    println!("stat                         Print a file's stats");
    println!("ls                           List files in a directory");
    println!("echo                         Print a message");
    println!("fork_test                    Run a fork and waitpid test");
    println!("thread_test                  Run a multi-thread test");
    println!("timeshard_test               Run a test to see Round-robin with TimeShards");
    println!("shell                        Open a new shell");
    return 0;
}
