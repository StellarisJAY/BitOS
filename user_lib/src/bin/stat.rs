#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;
use user_lib::file::{get_absolute_path, File, OpenFlags};

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    if argc <= 1 || argv[0].is_empty() {
        println!("[error] empty file name");
        return -1;
    }
    let name = String::from(argv[0]);
    let cur_path = String::from(argv[argc - 1]);
    let absolute_path = get_absolute_path(name, cur_path);
    let mut file_path = absolute_path.clone();
    file_path.push('\0');
    match File::open(file_path.as_str(), OpenFlags::RDONLY) {
        Ok(file) => {
            let stat = file.fstat().unwrap();
            println!("File:  {}", absolute_path);
            println!(
                "Type:  {}",
                if stat.dir {
                    "directory"
                } else {
                    "regular file"
                }
            );
            println!(
                "Size:  {:<16} Blocks:       {:<16} IO Block: {:<16}",
                stat.size, stat.blocks, stat.io_block
            );
            println!(
                "Inode: {:<16} Index Blocks: {:<16}",
                stat.inode, stat.index_blocks
            );
            file.close();
        }
        Err(_) => {
            println!("[error] File not found: {}", absolute_path);
        }
    }
    return 0;
}
