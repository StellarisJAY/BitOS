#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file::{self, get_absolute_path};

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc == 0 || argv[0].is_empty() {
        println!("[error] empty file name");
        return -1;
    }
    // argv[n-1]是固定参数：当前目录
    let relative = if argc == 1 {
        // ls 不传参时，相对路径为空
        String::from("")
    } else {
        String::from(argv[0])
    };
    let cur_path = String::from(argv[argc - 1]);
    let absolute_path = get_absolute_path(relative, cur_path.clone());
    let mut path = absolute_path.clone();
    path.push('\0');
    match file::ls(path.as_str()) {
        Ok(files) => {
            println!("{:>10}  {:4}  {:28}", "size", "type", "name");
            for f in files {
                let mut file_path = String::from(absolute_path.clone().trim_matches('\0'));
                if !file_path.ends_with('/') {
                    file_path.push('/');
                }
                file_path.push_str(f.trim_matches('\0'));
                file_path.push('\0');
                if let Some(stat) = file::stat(file_path.as_str()) {
                    let _type = if stat.dir { "dir" } else { "file" };
                    println!("{:10}  {:4}  {:28}", stat.size, _type, f);
                }
            }
        }
        Err(code) => return code as i32,
    }
    return 0;
}
