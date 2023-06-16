#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::string::String;

#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    let mut msg = String::new();
    for (i, arg) in argv.iter().enumerate() {
        msg.push_str(*arg);
        if i != argc - 1 {
            msg.push(' ');
        }
    }
    println!("{}", msg);
    return 0;
}
