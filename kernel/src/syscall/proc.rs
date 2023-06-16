use crate::fs::inode::{open_file, OpenFlags};
use crate::mem::address::VirtAddr;
use crate::mem::kernel;
use crate::proc::loader::load_kernel_app;
use crate::proc::pcb::{ProcessControlBlock, ProcessState};
use crate::task::scheduler::{
    add_process, current_proc, current_task, current_task_translate_buffer,
    current_task_trap_context, exit_current_task, push_task, remove_process, schedule_idle,
};
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

pub fn sys_spawn(ptr: usize, args: usize, args_count: usize) -> isize {
    let parent = current_proc();
    // 获取命令行参数数组地址
    let args_ptr = parent.translate_va(args) as *const usize;
    let mut args_vec: Vec<String> = Vec::new();
    unsafe {
        // 数组的每一个元素是&[u8]数组指针
        let args = core::slice::from_raw_parts(args_ptr, args_count);
        for (i, arg) in args.iter().enumerate() {
            // 将用户空间指针转换成内核空间字符串
            let arg_str = parent.translate_string(*arg);
            args_vec.push(arg_str);
        }
    }
    let app_name = parent.translate_string(ptr);
    // 文件系统加载app数据
    if let Some(file) = open_file(app_name.as_str(), OpenFlags::RDONLY) {
        let proc = ProcessControlBlock::from_elf_data(file.read_all().as_slice());
        let mut child_inner = proc.borrow_inner();
        let mut parent_inner = parent.borrow_inner();
        // 设置父子进程关系
        parent_inner.children.push(Arc::clone(&proc));
        child_inner.parent = Some(Arc::clone(&parent));
        drop(parent_inner);
        // 将命令行参数压入主线程用户栈
        let main_task = child_inner.tasks[0].as_ref();
        let main_trap_ctx = main_task.trap_context();
        drop(child_inner);
        // 为命令行参数分配栈空间，一个参数数量 + n个参数指针
        let mut user_sp = main_trap_ctx.sp;
        user_sp -= args_vec.len() * core::mem::size_of::<usize>();
        let args_ptr_base = user_sp;
        unsafe {
            // 收集用户栈上分配参数指针的地址
            let mut args_ptr: Vec<_> = (0..args_vec.len())
                .map(|i| proc.translate_va(user_sp + i * core::mem::size_of::<usize>()))
                .collect();

            for (i, arg) in args_vec.iter().enumerate() {
                // 为参数字符串分配连续地址
                user_sp -= arg.len() + 1;
                (args_ptr[i] as *mut usize).write_volatile(user_sp);
                // 将参数的每一个字节压入栈
                let mut ptr = user_sp;
                for b in arg.as_bytes() {
                    (proc.translate_va(ptr) as *mut u8).write_volatile(*b);
                    ptr += 1;
                }
                // 压入结尾的\0字符
                (proc.translate_va(ptr) as *mut u8).write_volatile(0);
            }
            // 修改主线程的sp和函数参数
            main_trap_ctx.sp = user_sp;
            main_trap_ctx.a[0] = args_vec.len(); // argc
            main_trap_ctx.a[1] = args_ptr_base; // argv
        }

        // 返回pid
        return proc.pid() as isize;
    } else {
        return -1;
    }
}
