#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use user_lib::{create_thread, wait_tid};

#[no_mangle]
pub fn main() -> i32 {
    let n = 3;
    println!("This time-shard test will create {} threads.", n);
    println!("Observe each thread's progress to see RoundRobin with TimeShards");
    
    let mut handles: Vec<isize> = Vec::new();
    let mut args_holder: Vec<Vec<usize>> = Vec::new();
    for i in 0usize..n {
        args_holder.push(vec![i, 10000]);
        let handle = create_thread(task as usize, &args_holder[i] as *const _ as usize);
        handles.push(handle);
    }
    
    for handle in handles {
        wait_tid(handle);
    }
    return 0;
}

fn task(args: &Vec<usize>) {
    let mut progress = 0;
    let id = args[0];
    for i in 0..args[1] {
        if i % 100 == 0 {
            println!("[t{}] progress: {}%", id, progress);
            progress += 1;
        }
    }
    println!("[t{}] finished", id);
    user_lib::exit(0);
}