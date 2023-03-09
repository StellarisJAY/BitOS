#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

mod syscall;
#[macro_use]
mod utils;

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
}

pub fn exit(code: i32) {
    syscall::exit(code);
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::write(fd, buf)
}
