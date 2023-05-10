use alloc::vec::Vec;
use lazy_static::lazy_static;

extern "C" {
    fn _app_names();
    fn _app_addrs();
}

lazy_static! {
    pub static ref APP_NAMES: Vec<&'static str> = unsafe {
        let count = get_app_count();
        let mut ptr = _app_names as usize as *mut u8;
        let mut names: Vec<&'static str> = Vec::new();
        for i in 0..count {
            let mut p = ptr;
            let mut length: usize = 0;
            while p.read_volatile() != b'\0' {
                p = p.add(1);
                length += 1;
            }
            let slice = core::slice::from_raw_parts(ptr, length);
            names.push(core::str::from_utf8(slice).unwrap());
            ptr = p.add(1);
        }
        names
    };
}

// 加载kernel地址空间中.data段中的内核应用程序elf数据
pub fn load_kernel_app(name: &str) -> Option<&[u8]> {
    let count = get_app_count();
    return APP_NAMES
        .iter()
        .enumerate()
        .find(|(_, n)| **n == name)
        .map(|(id, _)| {
            return load_app(id);
        });
}

fn load_app<'a>(id: usize) -> &'a [u8] {
    unsafe {
        let mut ptr = _app_addrs as usize as *mut usize;
        let start_addr = ptr.add(id + 1).read_volatile();
        let length = ptr.add(id + 2).read_volatile() - start_addr;
        return core::slice::from_raw_parts(start_addr as *const u8, length);
    }
}

fn get_app_count() -> usize {
    unsafe {
        let ptr = _app_addrs as usize as *const usize;
        return ptr.read_volatile();
    }
}
