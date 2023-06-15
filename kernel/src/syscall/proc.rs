use crate::proc::loader::load_kernel_app;
use crate::proc::pcb::{ProcessControlBlock, ProcessState};
use crate::task::scheduler::{
    add_process, current_task, current_task_translate_buffer, current_task_trap_context,
    exit_current_task, push_task, remove_process, schedule_idle,
};
use crate::fs::inode::{open_file, OpenFlags};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_exit(exit_code: i32) -> isize {
    exit_current_task(exit_code);
    schedule_idle();
    return 0;
}

pub fn sys_yield() -> isize {
    let task = current_task();
    push_task(task);
    schedule_idle();
    return 0;
}

pub fn sys_fork() -> usize {
    let task = current_task();
    let parent = task.inner.borrow().process.upgrade().unwrap();
    let child = ProcessControlBlock::fork(Arc::clone(&parent), task.tid);
    add_process(Arc::clone(&child));
    debug!("child pid: {}", child.pid());
    child.pid()
}

pub fn sys_waitpid(pid: usize) -> isize {
    let task = current_task();
    let current_proc = task.inner.borrow().process.upgrade().unwrap();
    let mut inner = current_proc.borrow_inner();
    let res = inner
        .children
        .iter()
        .enumerate()
        .find(|(_, child)| child.pid() == pid) // 从当前进程的子进程中找到pid
        .map(|(index, child)| {
            let child_inner = child.borrow_inner();
            if child_inner.status == ProcessState::Zombie {
                // 子进程是僵尸进程，删除pcb所有权，回收资源
                remove_process(child.pid());
                return (index, child_inner.exit_code as isize);
            } else {
                return (index, -2);
            }
        });
    if let Some((index, exit_code)) = res {
        if exit_code != -2 {
            inner.children.remove(index);
        }
        return exit_code;
    }
    0
}

pub fn sys_spawn(ptr: usize, len: usize) -> isize {
    // 获取app name
    let buf = current_task_translate_buffer(ptr, len);
    let mut bytes: Vec<u8> = Vec::new();
    for part in buf {
        bytes.extend_from_slice(part);
    }
    let app_name = String::from_utf8(bytes).unwrap();
    // 文件系统加载app数据
    if let Some(file) = open_file(app_name.as_str(), OpenFlags::RDONLY) {
        let proc = ProcessControlBlock::from_elf_data(file.read_all().as_slice());
        let cur_task = current_task();
        let task = cur_task.inner.borrow();
        let parent = task.process.upgrade().unwrap();
        let mut parent_inner = parent.borrow_inner();
        // 设置父子进程关系
        parent_inner.children.push(Arc::clone(&proc));
        proc.borrow_inner().parent = Some(Arc::clone(&parent));
        // 返回pid
        return proc.pid() as isize;
    }else {
        return -1;
    }
}
