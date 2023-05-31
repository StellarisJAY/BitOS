#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use user_lib::sync::mutex::Mutex;
use user_lib::time::get_time_ms;
use user_lib::sync::cond::Cond;

struct Point {
    x: isize,
    y: isize,
}

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time_ms();
    test_create_thread();
    test_mutex();
    test_cond();
    println!("Thread Test Finish. Time used: {} ms", get_time_ms() - start);
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
    let m = Mutex::new(true);
    let mut threads: Vec<isize> = Vec::new();
    let start = get_time_ms();
    for _ in 0..10 {
        let tid = user_lib::create_thread(mutex_func as usize, &m as *const _ as usize);
        threads.push(tid);
        println!("[main] thread-{} created", tid);
    }
    for tid in threads {
        user_lib::wait_tid(tid);
        println!("[main] thread-{} finished", tid);
    }
    unsafe {
        println!("Test Finished, time used: {} ms, Sum = {}", get_time_ms() - start, SUM);
        assert!(SUM == 100_000, "Mutex Teest Failed");
    }
}

fn test_cond() {
    let producer = user_lib::create_thread(producer_func as usize, 0);
    let consumer = user_lib::create_thread(consumer_func as usize, 0);
    user_lib::wait_tid(producer);
    user_lib::wait_tid(consumer);
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

fn mutex_func(m: &Mutex) {
    for _ in 0..10000 {
        m.lock();
        unsafe{SUM += 1;}
        m.unlock();
    }
    user_lib::exit(0);
}


use lazy_static::lazy_static;
static mut STORAGE: usize = 0;

lazy_static! {
    static ref MUTEX: Mutex = Mutex::new(false);
}

lazy_static! {
    static ref COND: Cond = Cond::new(&MUTEX);
}

fn producer_func() {
    MUTEX.lock();
    unsafe{
        STORAGE = 1;
        println!("[producer] send: {}", STORAGE);
    }
    MUTEX.unlock();
    user_lib::exit(0);
}

fn consumer_func() {
    MUTEX.lock();
    unsafe {
        while STORAGE == 0 {
            COND.wait();
        }
        println!("[consumer] recv: {}", STORAGE);
    }
    MUTEX.unlock();
    user_lib::exit(0);
}