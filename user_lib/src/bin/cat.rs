#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file::{File, OpenFlags};

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc == 0 {
        println!("[error] empty file name");
        return 0;
    }
    let name = String::from(argv[0]);
    let mut cur_path = String::from(argv[argc - 1]);
    let absolute_path = get_absolute_path(&name, &mut cur_path);

    if let Some(file) = File::open(absolute_path.as_str(), OpenFlags::RDONLY) {
        let mut buf: [u8; 512] = [0; 512];
        while file.read(&mut buf) != 0 {
            print!("{}", String::from_utf8_lossy(&buf));
            buf.fill(0);
        }
        println!("");
        file.close();
    } else {
        println!("[error] File not found: {}", name);
    }

    return 0;
}

fn get_absolute_path(name: &String, cur_path: &mut String) -> String {
    let mut absolute_path: String;
    if name.starts_with("/") {
        absolute_path = name.clone();
    }else {
        cur_path.push('/');
        cur_path.push_str(name.as_str());
        absolute_path = cur_path.clone();
    }
    absolute_path.push('\0');
    return absolute_path;
}