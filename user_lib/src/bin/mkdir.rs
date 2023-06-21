#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file::{get_absolute_path, File, OpenFlags, FILE_EXIST_ERROR, NOT_DIR_ERROR};

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc <= 1 || argv[0].is_empty() {
        println!("[error] empty file name");
        return -1;
    }
    let relative = String::from(argv[0]);
    let cur_path = String::from(argv[argc - 1]);
    let absolute_path = get_absolute_path(relative, cur_path);
    let mut file_path = absolute_path.clone();
    file_path.push('\0');
    match File::open(file_path.as_str(), OpenFlags::DIR | OpenFlags::CREATE) {
        Ok(file) => {
            file.close();
        }
        Err(code) => match code {
            FILE_EXIST_ERROR => {
                println!("cannot create directory '{}': File exists", absolute_path)
            }
            NOT_DIR_ERROR => println!("cannot create directory '{}': Not directory", absolute_path),
            _ => println!("fs error, code: {}", code),
        },
    }
    return 0;
}
