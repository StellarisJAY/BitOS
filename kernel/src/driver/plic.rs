use crate::arch::riscv::qemu::layout::{
    PLIC_SOURCES, UART0, UART_PLIC, VIRTIO_BLK_PLIC, VIRT_PLIC,
};
use crate::driver::uart;

#[allow(clippy::upper_case_acronyms)]
pub struct PLIC {
    base_addr: usize,
}

#[derive(Copy, Clone)]
pub enum IntrTargetPriority {
    Machine = 0,
    Supervisor = 1,
}

impl IntrTargetPriority {
    pub fn supported_number() -> usize {
        2
    }
}

pub fn init_plic() {
    use riscv::register::sie;
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let hart_id = 0usize;
    // Machine的阈值设为1，Supervisor阈值设为0，使中断优先级低于1的都由Supervisor处理
    plic.set_threshold(hart_id, IntrTargetPriority::Machine, 1);
    plic.set_threshold(hart_id, IntrTargetPriority::Supervisor, 0);

    // 中断源，uart：10，blk：8
    for src_id in PLIC_SOURCES {
        // cpu中断使能
        plic.enable(hart_id, IntrTargetPriority::Supervisor, *src_id);
        // 设置src中断的优先级为1
        plic.set_priority(*src_id, 1);
    }

    // Supervisor外设中断使能
    unsafe {
        sie::set_sext();
    }
}

pub fn handle_irq() {
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    // 读PLIC的 Claim 寄存器获得外设中断号
    let src = plic.claim(0, IntrTargetPriority::Supervisor);
    match src as usize {
        UART_PLIC => {
            uart::handle_irq();
        }
        VIRTIO_BLK_PLIC => {
            panic!("virtio blk over plic not implemented");
        }
        _ => panic!("unsupported IRQ {}", src),
    }
    // 中断完成
    plic.complete(0, IntrTargetPriority::Supervisor, src);
}

impl PLIC {
    fn priority_addr(&self, intr_source_id: usize) -> usize {
        assert!(intr_source_id > 0 && intr_source_id <= 132);
        self.base_addr + intr_source_id * 4
    }
    fn hart_id_with_priority(hart_id: usize, target_priority: IntrTargetPriority) -> usize {
        let priority_num = IntrTargetPriority::supported_number();
        hart_id * priority_num + target_priority as usize
    }
    fn enable_ptr(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        intr_source_id: usize,
    ) -> (*mut u32, usize) {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        let (reg_id, reg_shift) = (intr_source_id / 32, intr_source_id % 32);
        (
            (self.base_addr + 0x2000 + 0x80 * id + 0x4 * reg_id) as *mut u32,
            reg_shift,
        )
    }
    fn threshold_ptr_of_hart_with_priority(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
    ) -> *mut u32 {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        (self.base_addr + 0x20_0000 + 0x1000 * id) as *mut u32
    }
    fn claim_comp_ptr_of_hart_with_priority(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
    ) -> *mut u32 {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        (self.base_addr + 0x20_0004 + 0x1000 * id) as *mut u32
    }

    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
    // 设置一个中断来源的优先级
    pub fn set_priority(&mut self, src_id: usize, priority: u32) {
        assert!(priority < 8);
        unsafe {
            let ptr = self.priority_addr(src_id) as *mut u32;
            ptr.write_volatile(priority);
        }
    }
    // 中断使能
    pub fn enable(
        &mut self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        intr_source_id: usize,
    ) {
        let (reg_ptr, shift) = self.enable_ptr(hart_id, target_priority, intr_source_id);
        unsafe {
            reg_ptr.write_volatile(reg_ptr.read_volatile() | 1 << shift);
        }
    }
    // 设置特权级对应的优先级阈值，小于等于阈值的中断会屏蔽
    pub fn set_threshold(
        &mut self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        threshold: u32,
    ) {
        assert!(threshold < 8);
        let threshold_ptr = self.threshold_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe {
            threshold_ptr.write_volatile(threshold);
        }
    }

    // 获取当前中断claim
    pub fn claim(&mut self, hart_id: usize, target_priority: IntrTargetPriority) -> u32 {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe { claim_comp_ptr.read_volatile() }
    }

    // 特定中断处理完成
    pub fn complete(
        &mut self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        completion: u32,
    ) {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe {
            claim_comp_ptr.write_volatile(completion);
        }
    }
}
