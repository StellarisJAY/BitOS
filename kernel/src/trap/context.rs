
// TrapContext
// 陷入内核态后用来保存用户态寄存器
#[repr(C)]
pub struct TrapContext {
    pub kernel_satp: usize,  // 0   内核satp，用于恢复内核地址空间 (不变)
    pub kernel_sp: usize,    // 8   进程的内核栈sp （每个进程的内核栈指针固定不变）
    pub trap_handler: usize, // 16  trap处理器地址 （固定地址）
    pub sepc: usize,         // 24  用户空间pc
    pub sp: usize,           // 32  用户空间sp
    pub ra: usize,           // 40  返回地址
    pub t: [usize; 7],       // 48  t0~t6
    pub a: [usize; 8],       // 104 a0~a7
    pub s: [usize; 12],      // 168 s0~s11
}

impl TrapContext {
    pub fn empty() -> Self {
        return Self { kernel_satp: 0, kernel_sp: 0, trap_handler: 0, sepc: 0, sp: 0, ra: 0, t: [0; 7], a: [0; 8], s: [0; 12]};
    }
    pub fn user_trap_context(kernel_satp: usize, kernel_sp: usize, trap_handler: usize, app_entry: usize, user_sp: usize) -> Self {
        return Self {
            kernel_satp: kernel_satp,
            kernel_sp: kernel_sp,
            trap_handler: trap_handler,
            sepc: app_entry,
            sp: user_sp,
            ra: 0,
            t: [0; 7],
            a: [0; 8],
            s: [0; 12],
        }
    }
}

