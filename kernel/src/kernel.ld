OUTPUT_ARCH(riscv)
ENTRY(__entry)              # kernel entry point（.text.entry）
BASE_ADDRESS = 0x80000000;   # kernel基地址

SECTIONS
{
    . = BASE_ADDRESS;
    # kernel起始地址
    skernel = .;
    stext = .;
    .text : {
        *(.text.entry)    # entry段，kernel entry code
        strampoline = .;
        *(.text.tramp)    # trampoline 代码
        . = ALIGN(4K);
        *(.text, .text.*) # text段
    }
    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata, .rodata.*) # rodata段
    }
    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data, .data.*) # data段
    }
    . = ALIGN(4K);
    edata = .;
    sbss = .;
    .bss : {
        *(.bss.stack)   # bss段的boot stack
        *(.bss, .bss.*) # bss段
        *(.sbss, .sbss.*)
    }
    . = ALIGN(4K);
    ebss = .;
    # kernel 内存结束位置
    ekernel = .;
    /DISCARD/ : {
        *(.eh_frame)
    }
}