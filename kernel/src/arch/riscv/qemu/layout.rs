pub const UART0: usize = 0x10000000;
pub const UART_PLIC: usize = 10;

pub const CLINT0: usize = 0x02000000;
pub const CLINT_MTIME: usize = CLINT0 + 0xBFF8;
pub const CLINT_MTIMECMP: usize = CLINT0 + 0x4000;

pub const VIRTIO0: usize = 0x10001000;
pub const SECTOR_SIZE: usize = 512;
pub const VIRTIO_BLK_PLIC: usize = 8;

pub const SHUTDOWN0: usize = 0x10_0000;

pub const VIRT_PLIC: usize = 0x0c00_0000;

// MMIO地址范围
pub const MMIO: &[(usize, usize)] = &[
    (UART0, UART0 + 0x1000),
    (CLINT0, CLINT0 + 0xc000),
    (VIRTIO0, VIRTIO0 + 0x1000),
    (SHUTDOWN0, SHUTDOWN0 + 0x1000),
    (VIRT_PLIC, VIRT_PLIC + 0x30_0000),
];

// PLIC中断源列表
pub const PLIC_SOURCES: &[usize] = &[
    UART_PLIC,       // uart
    VIRTIO_BLK_PLIC, // virtio blk
];
