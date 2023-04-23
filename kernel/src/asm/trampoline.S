# this code is mapped in each memset's same virt page number
.section .text.tramp
# user ecall or exception triggers user_vec
.global _user_vec
_user_vec:
    # begining sscratch points to process's trap context
    # swap a0, sscratch
    # now a0 points to trap context, sscratch holds a0
    csrrw a0, sscratch, a0

    # save t0..t6
    sd t0, 48(a0)
    sd t1, 56(a0)
    sd t2, 64(a0)
    sd t3, 72(a0)
    sd t4, 80(a0)
    sd t5, 88(a0)
    sd t6, 96(a0)

    # save a1..a7
    sd a1, 112(a0)
    sd a2, 120(a0)
    sd a3, 128(a0)
    sd a4, 136(a0)
    sd a5, 144(a0)
    sd a6, 152(a0)
    sd a7, 160(a0)

    # save s0..s11
    sd s0, 168(a0)
    sd s1, 176(a0)
    sd s2, 184(a0)
    sd s3, 192(a0)
    sd s4, 200(a0)
    sd s5, 208(a0)
    sd s6, 216(a0)
    sd s7, 224(a0)
    sd s8, 232(a0)
    sd s9, 240(a0)
    sd s10, 248(a0)
    sd s11, 256(a0)

    # save user's a0
    csrr t0, sscratch
    sd t0, 104(a0)

    # save sepc
    csrr t0, sepc
    sd t0, 24(a0)
    # load trap_handler address
    ld t0, 16(a0)
    # sp to kernel stack
    ld sp, 8(a0)
    # change satp to kernel space
    # after sfence.vma a0 is no longer accessible
    ld t1, 0(a0)
    csrw satp, t1
    sfence.vma zero, zero
    # jump to trap_handler
    jr t0

.global _user_ret
_user_ret:
    # user_ret(trap_context, satp)
    # a0: trap_context, a1: user space satp
    # change satp to process's user space
    csrw satp, a1
    sfence.vma zero,zero
    # save user's a0 in sscratch temporarily
    ld t0, 104(a0)
    csrw sscratch, t0
    # set sepc to user program's pc
    ld t0, 24(a0)
    csrw sepc, t0
    
    # load t0..t6
    ld t0, 48(a0)
    ld t1, 56(a0)
    ld t2, 64(a0)
    ld t3, 72(a0)
    ld t4, 80(a0)
    ld t5, 88(a0)
    ld t6, 96(a0)

    # load a1..a7
    ld a1, 112(a0)
    ld a2, 120(a0)
    ld a3, 128(a0)
    ld a4, 136(a0)
    ld a5, 144(a0)
    ld a6, 152(a0)
    ld a7, 160(a0)

    # load s0..s11
    ld s0, 168(a0)
    ld s1, 176(a0)
    ld s2, 184(a0)
    ld s3, 192(a0)
    ld s4, 200(a0)
    ld s5, 208(a0)
    ld s6, 216(a0)
    ld s7, 224(a0)
    ld s8, 232(a0)
    ld s9, 240(a0)
    ld s10, 248(a0)
    ld s11, 256(a0)

    # load user stack sp
    ld sp, 32(a0)
    # swap a0 and sscratch
    # after swap, a0=saved a0, sscratch=trap_context
    # so that in _user_vec we can access trap_context by sscratch
    csrrw a0, sscratch, a0
    # return to user mode
    sret
