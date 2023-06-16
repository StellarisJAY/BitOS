#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::file;
use user_lib::file::{File, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    println!("This is a file system test");
    create_file();
    read_file();
    read_fstat();
    stat();
    return 0;
}

fn create_file() {
    let file = File::open("test_file\0", OpenFlags::CREATE);
    let write_len = file.write("hello world".as_bytes());
    println!("file write finished, len: {}", write_len);
    file.close();
}

fn read_file() {
    let file = File::open("test_file\0", OpenFlags::RDONLY);
    let mut data: Vec<u8> = Vec::new();
    let mut buf: [u8; 11] = [0; 11];
    let read_len = file.read(&mut buf);
    data.extend_from_slice(&buf);
    println!("file read finished, len: {}", read_len);
    println!("content: {:?}", String::from_utf8(data).unwrap());
    file.close();
}

fn read_fstat() {
    let file = File::open("test_file\0", OpenFlags::RDONLY);
    match file.fstat() {
        Some(stat) => println!("{:?}", stat),
        None => panic!("read fstat error"),
    }
    file.close();
}

fn stat() {
    match file::stat("hello_world\0") {
        Some(file_stat) => println!("{:?}", file_stat),
        None => panic!("stat error"),
    }
}
