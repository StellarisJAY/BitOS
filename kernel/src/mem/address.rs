use crate::config::{PAGE_SIZE};

const SV39_PA_BITS: usize = 56;  // 物理地址长度
const SV39_PPN_BITS: usize = 44; // 物理页号长度：44bits
const SV39_VA_BITS: usize = 39;  // 虚拟内存长度：39bits，最大512GiB内存
const SV39_VPN_BITS: usize = 27; // 虚拟页号 27bits，三级页表每级9bits，最大寻址：2^27个物理页
const SV39_OFF_BITS: usize = 12; // 页内偏移 12bits，共4KiB范围

#[derive(Clone, Copy)]
pub struct PhysPageNumber (pub usize);
#[derive(Clone, Copy)]
pub struct PhysAddr (pub usize);
#[derive(Clone, Copy)]
pub struct VirtPageNumber (pub usize);
#[derive(Clone, Copy)]
pub struct VirtAddr (pub usize);

impl PhysPageNumber {
    // 物理页的起始地址
    pub fn base_addr(&self) -> usize {
        return self.0 << SV39_OFF_BITS;
    }
    // 获取物理页的数据
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let ptr = self.base_addr() as *const u8;
            return core::slice::from_raw_parts(ptr, PAGE_SIZE);
        }
    }

    // 将物理页从off开始转换成T类型，然后用f函数对T进行修改
    pub fn modify<T: Sized>(&self, off: usize, f: impl FnOnce(&mut T)){
        if off < PAGE_SIZE {
            unsafe {
                let ptr = (self.base_addr() + off) as *mut T;
                let s = ptr.as_mut().unwrap();
                f(s);
            }
        }
    }
}

impl PhysAddr {
    // 物理地址所在的物理页号
    pub fn page_number(&self) -> PhysPageNumber {
        return PhysPageNumber(self.0 >> SV39_OFF_BITS);
    }
    // 物理地址向上取整的物理页号
    pub fn ceil(&self) -> PhysPageNumber {
        return PhysPageNumber((self.0 >> SV39_OFF_BITS) + 1);
    }
}

