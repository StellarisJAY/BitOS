#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{fork, yield_};
#[no_mangle]
pub fn main() -> i32 {
    println!("[parent] fork test begin");
    let p1 = fork();
    if p1 == 0 {
        println!("[child] i am child 1");
        return 0;
    }else {
        println!("[parent] child1 pid: {}", p1);
        println!("[parent] i yield");
        yield_();
        println!("[parent] parent recovered");
        let p2 = fork();
        if p2 == 0 {
            println!("[child] i am child2");
            println!("[child] i yield");
            println!("[child] child2 recovered");
        }else {
            println!("[parent] child2 pid: {}", p2);
        }
    }
    return 0;
}