
// 进程上下文，切换进程时用来保存通用寄存器
#[repr(C)]
pub struct ProcessContext {
    pub ra: usize,           // ra函数返回地址寄存器
    pub sp: usize,           // 用户空间栈sp
    pub s: [usize; 12],      // s0~s11
}

impl ProcessContext {
    pub fn empty() -> Self {
        return Self { ra: 0, sp: 0, s: [0;12] };
    }
}