#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;

#[no_mangle]
pub fn main() -> i32 {
    println!("This is thread test");
    let t1 = user_lib::create_thread(thread1_func as usize, 0);
    let t2 = user_lib::create_thread(thread2_func as usize, 0);
    println!("thread created: {}, {}", t1, t2);
    
    println!("t1 done, exit code: {}", user_lib::wait_tid(t1));
    println!("t2 done, exit_code: {}", user_lib::wait_tid(t2));
    return 0;
}
#[no_mangle]
fn thread1_func() {
    println!("[t1] i am thread1");
    let mut nums: Vec<isize> = Vec::new();
    for i in 0..10 {
        nums.push(i);
    }
    println!("[t1] numbers: {:?}", nums.as_slice());
    user_lib::exit(10);
}

#[no_mangle]
fn thread2_func() {
    println!("[t2] i am thread2");
    let mut sum = 0;
    for i in 0..100 {
        sum += i;
    }
    println!("[t2] sum: {}", sum);
    user_lib::exit(2);
}