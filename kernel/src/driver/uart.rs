use crate::arch::riscv::qemu::layout::UART0;
use alloc::collections::VecDeque;
use core::fmt::*;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

// uart 寄存器组，see：https://www.lammertbies.nl/comm/info/serial-uart
const RHR: usize = 0; // 读缓冲（8bit）
const THR: usize = 0; // 写缓冲
const IER: usize = 1; // Interupt Enable
const FCR: usize = 2; // FIFO control
const LCR: usize = 3; // line control
const LSR: usize = 5; // line status

const DLL: usize = 0; // DLL, divisor latch LSB
const DLM: usize = 1; // DLM, divisor latch LMB

const FCR_FIFO_ENABLE: usize = 1 << 0;
const FCR_FIFO_CLEAR: usize = 3 << 1;
const LCR_EIGHT_BITS: usize = 3 << 0; // no parity
const LCR_BAUD_LATCH: usize = 1 << 7; // DLAB, DLL DLM accessible

const IER_RX_ENABLE: usize = 1 << 0;
const IER_TX_ENABLE: usize = 1 << 1;

pub struct Uart {
    recv_buf: VecDeque<u8>,
}

lazy_static! {
    pub static ref UART: Mutex<Uart> = Mutex::new(Uart::new());
}

// 从console读取一个字节，直接从写缓冲中读取
pub fn get_char() -> Option<u8> {
    let mut uart = UART.lock();
    let ch = uart.get_from_buf();
    drop(uart);
    return ch;
}

// uart中断处理，接收字节，添加到recv缓冲
pub fn handle_irq() {
    let mut uart = UART.lock();
    if let Some(ch) = uart.get() {
        uart.recv_buf.push_back(ch);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart {
    pub fn new() -> Self {
        Self {
            recv_buf: VecDeque::new(),
        }
    }
    pub fn init() {
        // 关闭中断
        write_reg(IER, 0x0);
        // DLAB
        write_reg(LCR, LCR_BAUD_LATCH as u8);
        // 38.4k baud rate, see:
        write_reg(DLL, 0x03);
        write_reg(DLM, 0x00);
        // 8 bits payload，无奇偶校验
        write_reg(LCR, LCR_EIGHT_BITS as u8);
        // 开启FIFO
        write_reg(FCR, FCR_FIFO_ENABLE as u8 | FCR_FIFO_CLEAR as u8);
        // enable transmit and receive interrupts.
        write_reg(IER, IER_TX_ENABLE as u8);
    }

    pub fn put(&self, ch: u8) {
        let ptr = reg_addr(THR) as *mut u8;
        loop {
            // 等待THR空闲
            if read_reg(LSR) & (1 << 5) != 0 {
                break;
            }
        }
        unsafe {
            ptr.write_volatile(ch);
        }
    }

    pub fn get(&self) -> Option<u8> {
        let ptr = reg_addr(RHR) as *mut u8;
        // 判断RHR是否有数据
        if read_reg(LSR) & 1 != 0 {
            unsafe {
                return Some(ptr.read_volatile());
            }
        } else {
            return None;
        }
    }

    pub fn get_from_buf(&mut self) -> Option<u8> {
        self.recv_buf.pop_front()
    }
}

fn reg_addr(reg: usize) -> usize {
    return UART0 + reg;
}

fn write_reg(reg: usize, val: u8) {
    let addr = reg_addr(reg);
    unsafe {
        let ptr = addr as *mut u8;
        (*ptr) = val;
    }
}

fn read_reg(reg: usize) -> u8 {
    let ptr = reg_addr(reg) as *const u8;
    unsafe { ptr.read_volatile() }
}
