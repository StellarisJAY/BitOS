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
        return -1;
    }
    let mut name = String::new();
    name.push_str(argv[0]);
    name.push('\0');

    if let Some(file) = File::open(name.as_str(), OpenFlags::RDONLY) {
        let stat = file.fstat().unwrap();
        println!("File:  {}", argv[0]);
        println!("Size:  {:<16} Blocks:       {:<16} IO Block: {:<16}", stat.size, stat.blocks, stat.io_block);
        println!("Inode: {:<16} Index Blocks: {:<16}", stat.inode, stat.index_blocks);
        file.close();
    } else {
        println!("[error] File not found: {}", name);
    }

    return 0;
}
