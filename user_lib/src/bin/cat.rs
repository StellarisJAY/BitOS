#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file::{File, OpenFlags, get_absolute_path};

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc <= 1 || argv[0].is_empty() {
        println!("[error] empty file name");
        return -1;
    }
    let name = String::from(argv[0]);
    let cur_path = String::from(argv[argc - 1]);
    let absolute_path = get_absolute_path(name.clone(), cur_path.clone());

    let mut file_path = absolute_path.clone();
    file_path.push('\0');
    match File::open(file_path.as_str(), OpenFlags::RDONLY) {
        Ok(file) => {
            let mut buf: [u8; 512] = [0; 512];
            while file.read(&mut buf) != 0 {
                print!("{}", String::from_utf8_lossy(&buf));
                buf.fill(0);
            }
            println!("");
            file.close();
        },
        Err(_) => {
            println!("[error] File not found: {}", absolute_path);
        }
    }
    return 0;
}