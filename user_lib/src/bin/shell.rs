#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
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
                
                // 按空格拆分字符串
                let parts: Vec<_> = cmd.split_whitespace().collect();
                // 末尾添加\0
                let mut str_parts: Vec<_> = parts
                    .iter()
                    .map(|arg| {
                        let mut arg_string = String::from(*arg);
                        arg_string.push('\0');
                        return arg_string;
                    })
                    .collect();
                let app = str_parts.remove(0);
                // args转换为指针数组
                let args: Vec<_> = str_parts.iter().map(|part| (*part).as_ptr()).collect();

                if let Some(pid) = user_lib::spawn(app.as_str(), args.as_slice()) {
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