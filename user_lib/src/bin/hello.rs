#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
#[no_mangle]
pub fn main(argc: usize, argv: &[&'static str]) -> i32 {
    println!("Hello world");
    println!("argc: {}", argc);
    for (i, arg) in argv.iter().enumerate() {
        println!("arg[{}] = {}", i, *arg);
    }
    return 0;
}
