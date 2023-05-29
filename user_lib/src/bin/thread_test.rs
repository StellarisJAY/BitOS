#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use user_lib::sync::mutex::Mutex;

struct Point {
    x: isize,
    y: isize,
}

#[no_mangle]
pub fn main() -> i32 {
    test_create_thread();
    test_mutex();
    return 0;
}

fn test_create_thread() {
    println!("This is Create Thread test");
    let point = Point{x: 10, y: 12};
    let nums: Vec<usize> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let t1 = user_lib::create_thread(thread1_func as usize, &point as *const _ as usize);
    let t2 = user_lib::create_thread(thread2_func as usize, &nums as *const _ as usize);
    println!("Thread Created: {}, {}", t1, t2);
    println!("t1 done, exit code: {}", user_lib::wait_tid(t1));
    println!("t2 done, exit_code: {}", user_lib::wait_tid(t2));
    println!("Create Thread Test Success\n");
}

fn test_mutex() {
    println!("This is Mutex Test");
    let m = Mutex::new(false);
    let t1 = user_lib::create_thread(mutex_func1 as usize, &m as *const _ as usize);
    let t2 = user_lib::create_thread(mutex_func2 as usize, &m as *const _ as usize);
    println!("thread created: {}, {}", t1, t2);
    println!("t1 done, exit code: {}", user_lib::wait_tid(t1));
    println!("t2 done, exit_code: {}", user_lib::wait_tid(t2));
    unsafe {
        println!("sum: {}", SUM);
    }
    println!("Mutex Test Sucess");
}

#[no_mangle]
fn thread1_func(p: &Point) {
    println!("[t1] i am thread1");
    println!("[t1] point: ({}, {})", p.x, p.y);
    let mut nums: Vec<isize> = Vec::new();
    for i in 0..10 {
        nums.push(i);
    }
    println!("[t1] numbers: {:?}", nums.as_slice());
    user_lib::exit(10);
}

#[no_mangle]
fn thread2_func(nums: &Vec<usize>) {
    println!("[t2] i am thread2");
    let mut sum = 0;
    for i in nums {
        sum += i;
    }
    println!("[t2] sum: {}", sum);
    user_lib::exit(2);
}

static mut SUM: usize = 0;

#[no_mangle]
fn mutex_func1(m: &Mutex) {
    println!("[t1] wait lock");
    m.lock();
    println!("[t1] acquired lock");
    user_lib::yield_();
    unsafe {
        for _ in 0..1000 {
            SUM += 1;
        }
    }
    user_lib::yield_();
    m.unlock();
    println!("[t1] exit");
    user_lib::exit(0);
}

#[no_mangle]
fn mutex_func2(m: &Mutex) {
    println!("[t2] wait lock");
    m.lock();
    println!("[t2] acquired lock");
    unsafe {
        for _ in 0..1000 {
            SUM += 1;
        }
    }
    m.unlock();
    println!("[t2] exit");
    user_lib::exit(0);
}