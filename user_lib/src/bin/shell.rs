#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use user_lib::file::{get_absolute_path, FILE_NOT_FOUND_ERROR, NOT_DIR_ERROR};
use user_lib::sync::cell::SafeCell;
use user_lib::utils::{get_char, put_char};
use user_lib::{spawn, wait_pid};

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const BS: u8 = 0x8;
const DL: u8 = 0x7f;

// shell builtin 命令集合
lazy_static! {
    static ref BUILTINS: SafeCell<BTreeSet<String>> = SafeCell::new(BTreeSet::new());
}

fn init() {
    let mut builtins = BUILTINS.borrow_inner();
    builtins.insert(String::from("cd"));
    builtins.insert(String::from("type"));
}

#[no_mangle]
pub fn main() -> i32 {
    init();
    // 命令程序所在的目录
    let mut app_absolute_path = String::from("/bin/");
    // shell当前所在的目录
    let mut cur_path = String::from("/");
    println!("User shell entered, input \"help\" to list available commands...");
    print!("\x1b[92mshell\x1b[0m:\x1b[94m{}\x1b[0m$ ", cur_path);
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
                let mut args: Vec<_> = cmd
                    .split_whitespace()
                    .map(|arg| String::from(arg))
                    .collect();
                // 将shell当前的目录添加到参数列表末尾
                args.push(cur_path.clone());
                // 获取要执行的命令
                let app = args.remove(0);
                match app.as_str() {
                    "cd" => cur_path = exec_cd(args, &mut cur_path),
                    "type" => exec_type(args, &mut app_absolute_path),
                    _ => _ = exec_app(args, app.clone(), &mut app_absolute_path),
                }

                cmd.clear();
                print!("\x1b[92mshell\x1b[0m:\x1b[94m{}\x1b[0m$ ", cur_path);
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

// 将参数转换成\0结尾的C字符串
fn process_args(args: Vec<String>) -> Vec<String> {
    return args
        .iter()
        .map(|arg| {
            let mut c_arg = arg.clone();
            c_arg.push('\0');
            return c_arg;
        })
        .collect();
}

fn exec_cd(args: Vec<String>, cur_path: &mut String) -> String {
    if args.len() <= 1 {
        println!("[error] empty path");
        return cur_path.clone();
    }
    let args = process_args(args);
    let path = args[0].as_str();
    let abs = get_absolute_path(String::from(path), cur_path.clone());
    let mut file_path = abs.clone();
    // 末尾插入\0
    file_path.push('\0');
    match File::open(file_path.as_str(), OpenFlags::RDONLY) {
        Ok(file) => {
            if !file.is_dir() {
                println!("[error] Not a directory: {}", abs);
                return cur_path.clone();
            }
            return abs;
        }
        Err(code) => {
            match code {
                NOT_DIR_ERROR => println!("[error] Not a directory: {}", abs),
                FILE_NOT_FOUND_ERROR => println!("File not found: {}", abs),
                _ => println!("[error] fs error, code: {}", code),
            }
            return cur_path.clone();
        }
    }
}

fn exec_app(args: Vec<String>, app: String, abs_path: &mut String) -> isize {
    let length = abs_path.len();
    abs_path.push_str(app.as_str());
    abs_path.push('\0');

    let c_args = process_args(args);
    let args_ptrs: Vec<_> = c_args.iter().map(|arg| (*arg).as_ptr()).collect();
    let code: isize;
    if let Some(pid) = spawn(abs_path.as_str(), args_ptrs.as_slice()) {
        code = wait_pid(pid);
    } else {
        println!("command not found");
        code = 0;
    }
    abs_path.truncate(length);
    return code;
}

fn exec_type(args: Vec<String>, abs_path: &mut String) {
    let builtins = BUILTINS.borrow_inner();
    for (i, cmd) in args.iter().enumerate() {
        if i == args.len() - 1 {
            continue;
        }
        if builtins.contains(cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            let length = abs_path.len();
            abs_path.push_str(cmd.as_str());
            abs_path.push('\0');
            match File::open(abs_path.as_str(), OpenFlags::RDONLY) {
                Ok(file) => {
                    println!("{} is {}", cmd, abs_path);
                    file.close();
                }
                Err(_) => println!("type: {} not found", cmd),
            }
            abs_path.truncate(length);
        }
    }
}
