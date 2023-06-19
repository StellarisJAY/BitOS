use crate::fs::inode::{find, open_file, OSInode, OpenFlags};
use crate::fs::FileStat;
use crate::fs::UserBuffer;
use simplefs::vfs::DIR_NAME_LIMIT;
use crate::task::scheduler::{current_proc, current_task_translate_string};
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let mut buf = UserBuffer::from_current_proc(buf_ptr, len);
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.write(&mut buf) as isize;
    }
    0
}

pub fn sys_read(fd: usize, buf_ptr: usize, len: usize) -> isize {
    let mut buf = UserBuffer::from_current_proc(buf_ptr, len);
    let proc = current_proc();
    let inner_pcb = proc.borrow_inner();
    if let Some(fd) = inner_pcb.fd_table[fd].as_ref() {
        let fd = Arc::clone(fd);
        drop(inner_pcb);
        return fd.read(&mut buf) as isize;
    }
    0
}

pub fn sys_open(path: usize, flags: u32) -> isize {
    let proc = current_proc();
    let name = proc.translate_string(path);
    if let Some(file) = open_file(name.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let fd = proc.alloc_fd();
        proc.borrow_inner().fd_table[fd] = Some(file);
        return fd as isize;
    } else {
        return -1;
    }
}

pub fn sys_close(fd: usize) -> isize {
    let proc = current_proc();
    let mut inner = proc.borrow_inner();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_stat(path: usize, stat: usize) -> isize {
    let proc = current_proc();
    let name = proc.translate_string(path);
    let file_stat: &mut FileStat;
    unsafe {
        let ptr = proc.translate_va(stat) as *mut FileStat;
        file_stat = ptr.as_mut().unwrap();
    }
    if let Some(inode) = find(name.as_str()) {
        inode.read_stat(file_stat);
        return 0;
    }
    return -1;
}

pub fn sys_fstat(fd: usize, stat: usize) -> isize {
    let proc = current_proc();
    let file_stat: &mut FileStat;
    unsafe {
        let ptr = proc.translate_va(stat) as *mut FileStat;
        file_stat = ptr.as_mut().unwrap();
    }
    let inner = proc.borrow_inner();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    let file = inner.fd_table[fd].as_ref().unwrap();
    if let Some(fstat) = file.fstat() {
        *file_stat = fstat;
        return 0;
    }
    return -1;
}


pub fn sys_lseek(fd: usize, offset: u32, from: u8) -> isize {
    let proc = current_proc();
    let inner = proc.borrow_inner();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -1;
    }
    let file = inner.fd_table[fd].as_ref().unwrap();
    return file.lseek(offset, from);
}

pub fn sys_ls_dir(path_ptr: usize, res: usize, size: usize) -> isize {
    let proc = current_proc();
    let name = proc.translate_string(path_ptr);
    if let Some(file) = open_file(name.as_str(), OpenFlags::RDONLY) {
        if !file.is_dir() {
            return -2;
        }
        let files = file.ls().unwrap();
        unsafe {
            let res_ptr = proc.translate_va(res) as *const usize;
            let result = core::slice::from_raw_parts(res_ptr, size);
            for (i, ptr) in result.iter().enumerate() {
                let addr = proc.translate_va(*ptr);
                let name = core::slice::from_raw_parts_mut(addr as *mut u8, DIR_NAME_LIMIT + 1);
                let fname_bytes = files[i].as_bytes();
                &mut name[0..fname_bytes.len()].copy_from_slice(fname_bytes);
            }
        }
        return 0;
    } else {
        return -1;
    }
    0
}