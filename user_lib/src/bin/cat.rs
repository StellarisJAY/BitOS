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
    let mut name = String::new();
    name.push_str(argv[0]);
    name.push('\0');

    if let Some(file) = File::open(name.as_str(), OpenFlags::RDONLY) {
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
