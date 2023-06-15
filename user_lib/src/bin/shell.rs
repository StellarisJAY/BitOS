#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::utils::{get_char, put_char};

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const BS: u8 = 0x8;
const DL: u8 = 0x7f;

#[no_mangle]
pub fn main() -> i32 {
    println!("User shell entered, input \"help\" to list available commands...");
    print!(">>> ");
    let mut cmd = String::new();
    loop {
        let byte = get_char();
        match byte {
            0 => continue,
            BS | DL => {
                print!("{} {}", BS as char, BS as char);
                cmd.pop();
            }
            CR | LF => {
                if cmd == "exit" {
                    println!("\nshell exited");
                    break;
                }
                put_char(b'\n');
                if let Some(pid) = user_lib::spawn(cmd.as_str()) {
                    user_lib::wait_pid(pid);
                } else {
                    println!("command not found");
                }
                cmd.clear();
                print!(">>> ");
            }
            _ => {
                put_char(byte);
                cmd.push(byte as char);
            }
        }
    }
    return 0;
}
