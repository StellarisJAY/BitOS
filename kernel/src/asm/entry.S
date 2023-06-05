.section .text.entry  # 启动代码在.text.entry
.global __entry

__entry:
    # sp = boot栈低（低地址）
    la sp, boot_stack_bottom
    # 读取当前的cpu核心id到a1
    csrr a1, mhartid
    # 第i个cpu的栈顶 = stack_bottom + (i + 1) * 8192
    li a0, 16384
    addi a1, a1, 1
    mul a1, a1, a0
    add sp, sp, a1
    # jump to rust_start()
    call rust_start

# 初始化boot栈，每个cpu拥有8KiB
# 栈位于bss段的stack段
.section .bss.stack
.align 4
.global boot_stack_bottom
boot_stack_bottom:
    .space 16384 * 4
    .global boot_stack_top
boot_stack_top:
