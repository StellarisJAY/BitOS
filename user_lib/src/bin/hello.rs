#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
#[no_mangle]
pub fn main() -> i32 {
    println!("Hello world");
    user_lib::yield_();
    println!("Hello world again");
    return 0;
}