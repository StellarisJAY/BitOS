pub const CPUS: usize = 1;
pub const PHYS_MEM_LIMIT: usize = 0x88000000; // 内存大小=2GB
pub const KERNEL_HEAP_SIZE: usize = 0x00100000;
pub const PAGE_SIZE: usize = 4096;

pub const TIME_FREQ: usize = 10000000;

// 每个进程的用户态栈大小：
pub const USER_STACK_SIZE: usize = 8 * 1024;
// 每个进程的内核栈大小：4KiB
pub const KERNEL_STACK_SIZE: usize = 4 * 1024;
// 虚拟地址最大值
pub const MAX_VA: usize = 4 << 30;
pub const TRAMPOLINE: usize = MAX_VA - PAGE_SIZE;
// 守护页，在栈的底端加上一个不被页表映射的页，栈溢出时会触发Pagefault，方便捕获栈溢出异常
pub const GUARD_PAGE: usize = PAGE_SIZE;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

// 最大pid值
pub const MAX_PID: usize = 1 << 15;
// 最大的内核栈数量
pub const MAX_KSTACKS: usize = 1024;
// 内核栈区域底部地址
pub const KERNEL_STACK_BOTTOM: usize = TRAMPOLINE - MAX_KSTACKS * (KERNEL_STACK_SIZE + GUARD_PAGE);
// 最大线程数量
pub const MAX_THREADS: usize = 64;
// 每个应用程序地址空间的trap区域基址
pub const TRAP_CONTEXT_BOTTOM: usize = TRAMPOLINE - MAX_THREADS * PAGE_SIZE;
// 是否显示debug信息
pub const DEBUG_MODE: bool = false;
// 是否启用时间片
pub const ENABLE_TIMER: bool = false;

// 获取线程在用户空间的trap_context地址
pub fn task_trap_context_position(tid: usize) -> usize {
    TRAMPOLINE - (tid + 1) * PAGE_SIZE
}

// 获取线程在用户空间的栈位置
pub fn task_user_stack_position(user_stack_bottom: usize, tid: usize) -> (usize, usize) {
    let stack_bottom = user_stack_bottom + tid * USER_STACK_SIZE;
    let stack_top = stack_bottom + USER_STACK_SIZE;
    return (stack_bottom, stack_top);
}
