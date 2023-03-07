OUTPUT_ARCH(riscv)
ENTRY(__entry)              # kernel entry point（.text.entry）
BASE_ADDRESS = 0x80000000;   # kernel基地址

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)    # entry段，kernel entry code
        *(.text, .text.*) # text段
    }
    . = ALIGN(4K);

    .rodata : {
        *(.rodata, .rodata.*) # rodata段
    }
    . = ALIGN(4K);

    .data : {
        *(.data, .data.*) # data段
    }
    . = ALIGN(4K);

    sbss = .;
    .bss : {
        *(.bss.stack)   # bss段的boot stack
        *(.bss, .bss.*) # bss段
    }
    ebss = .;
    . = ALIGN(4K);

    /DISCARD/ : {
        *(.eh_frame)
    }
}