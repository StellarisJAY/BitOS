#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

mod syscall;
#[macro_use]
pub mod utils;
pub mod sync;
pub mod time;
pub mod file;

const USER_HEAP_SIZE: usize = 4096 * 1024;

#[global_allocator]
static HEAP: buddy_system_allocator::LockedHeap = buddy_system_allocator::LockedHeap::new();
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

fn init_heap() {
    unsafe {
        let mut heap = HEAP.lock();
        heap.init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
        drop(heap);
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    init_heap();
    exit(main());
}

#[linkage="weak"]
#[no_mangle]
fn main() ->i32 {
    panic!("no main found")
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> !{
    match info.location() {
        Some(loc) => {
            if let Some(msg) = info.message() {
                println!("app panicked at {}:{}, message: {}",loc.file(), loc.line(), msg.as_str().unwrap());
            }else {
                println!("app panicked at {}:{}",loc.file(), loc.line());
            }
        },
        None => {
            println!("app panicked");
        }
    }
    exit(-1);
    loop{}
}

pub fn exit(code: i32) {
    syscall::exit(code);
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::write(fd, buf)
}

pub fn read(fd: usize, buf: &[u8]) -> isize {
    syscall::read(fd, buf)
}

pub fn fork() -> isize {
    syscall::fork()
}

pub fn yield_() {
    syscall::yield_();
}

pub fn spawn(name: &str) -> Option<usize> {
    let buf = name.as_bytes();
    let res = syscall::spawn(buf);
    if res == -1 {
        return None;
    }
    return Some(res as usize);
}

pub fn wait_pid(pid: usize) -> isize {
    loop {
        match syscall::wait_pid(pid) {
            -2 => yield_(),
            exit_code => return exit_code,
        }
    }
}

pub fn create_thread(entry: usize, args: usize) -> isize {
    syscall::create_thread(entry, args)
}

pub fn wait_tid(tid: isize) -> isize {
    loop {
        match syscall::wait_tid(tid) {
            -2 => yield_(),
            exit_code => return exit_code,
        }
    }
}

pub fn mutex_create(blocking: bool) -> isize {
    syscall::mutex_create(blocking)
}

pub fn mutex_lock(id: isize) {
    syscall::mutex_lock(id);
}

pub fn mutex_unlock(id: isize) {
    syscall::mutex_unlock(id);
}

pub fn cond_create() -> isize {
    syscall::cond_create()
}

pub fn cond_signal(id: isize) -> isize {
    syscall::cond_signal(id)
}

pub fn cond_wait(id: isize, mutex: isize) -> isize {
    syscall::cond_wait(id, mutex)
}