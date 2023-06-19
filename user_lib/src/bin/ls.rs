#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file;

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc == 0 || argv[0].is_empty() {
        println!("[error] empty file name");
        return -1;
    }
    let absolute_path: String;
    if argc == 1 {
        absolute_path = String::from(argv[0]);
    }else {
        let name = String::from(argv[0]);
        let mut cur_path = String::from(argv[argc - 1]);
        absolute_path = get_absolute_path(&name, &mut cur_path);
    }

    match file::ls(absolute_path.as_str()) {
        Ok(files) => {
            for f in files {
                println!("{:28}", f);
            }
        },
        Err(code) => return code as i32,
    }
    return 0;
}

fn get_absolute_path(name: &String, cur_path: &mut String) -> String {
    let mut absolute_path: String;
    if name.starts_with("/") {
        absolute_path = name.clone();
    }else {
        if cur_path.as_bytes()[cur_path.len() - 1] != b'/' {
            cur_path.push('/');
        }
        cur_path.push_str(name.as_str());
        absolute_path = cur_path.clone();
    }
    absolute_path.push('\0');
    return absolute_path;
}