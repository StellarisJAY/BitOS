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
    let mut app_absolute_path = String::from("/bin/");
    let mut cur_path = String::from("/");
    println!("User shell entered, input \"help\" to list available commands...");
    print!("{} >>> ", cur_path);
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
                let mut parts: Vec<_> = cmd.split_whitespace().collect();
                parts.push(cur_path.as_str());
                // 末尾添加\0
                let mut str_parts: Vec<_> = parts
                    .iter()
                    .map(|arg| {
                        let mut arg_string = String::from(*arg);
                        arg_string.push('\0');
                        return arg_string;
                    })
                    .collect();
                let app: String = str_parts.remove(0);
                let path_length: usize = app_absolute_path.len();
                if app == "cd\0" {
                    cd(str_parts, &mut cur_path);
                }else {
                    app_absolute_path.push_str(app.as_str());
                    // args转换为指针数组
                    let args: Vec<_> = str_parts.iter().map(|part| (*part).as_ptr()).collect();
                    if let Some(pid) = user_lib::spawn(app_absolute_path.as_str(), args.as_slice()) {
                        user_lib::wait_pid(pid);
                    } else {
                        println!("command not found");
                    }
                    app_absolute_path.truncate(path_length);
                }
                cmd.clear();
                print!("{} >>> ", cur_path);
            }
            _ => {
                put_char(byte);
                cmd.push(byte as char);
            }
        }
    }
    return 0;
}

use user_lib::file::{File, OpenFlags};

fn cd(args: Vec<String>, cur_path: &mut String) {
    if args.len() < 1 {
        println!("[error] empty path");
        return;
    }
    let path = args[0].as_str();
    let length = cur_path.len();
    cur_path.push_str(path);
    if let Some(file) = File::open(cur_path.as_str(), OpenFlags::RDONLY) {
        if !file.is_dir() {
            println!("Not a directory: {}", path);
            cur_path.truncate(length);
            return;
        }
        cur_path.pop();
        return;
    }else {
        println!("can't open directory: {}", path);
        cur_path.truncate(length);
        return;
    }
}