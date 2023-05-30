.section .text
.global timervec
timervec:
    # now a0 points to scratch[mhartid]
    csrrw a0, mscratch, a0
    # save registers
    sd a1, 0(a0)
    sd a2, 8(a0)
    sd a3, 16(a0)

    # rewrite a1,a3,a4 to prepare next timer interrupt
    ld a1, 24(a0) # MTIMECMP addr
    ld a2, 32(a0) # write timer interval to a2
    ld a3, 0(a1)  # read mtimecmp
    # interval + mtimecmp
    add a3, a3, a2
    # write mtimecmp
    sd a3, 0(a1)

    # trigger a supervisor interrupt
    li a1, 2
    csrw sip, a1
   
    ld a3, 16(a0)
    ld a2, 8(a0)
    ld a1, 0(a0)
    csrrw a0, mscratch, a0
    mret

.global _kernel_vec
_kernel_vec:
    # make space for saving registers
    addi sp, sp, -264

    # save t0..t6
    sd t0, 48(sp)
    sd t1, 56(sp)
    sd t2, 64(sp)
    sd t3, 72(sp)
    sd t4, 80(sp)
    sd t5, 88(sp)
    sd t6, 96(sp)

    # save a1..a7
    sd a1, 112(sp)
    sd a2, 120(sp)
    sd a3, 128(sp)
    sd a4, 136(sp)
    sd a5, 144(sp)
    sd a6, 152(sp)
    sd a7, 160(sp)

    # save s0..s11
    sd s0, 168(sp)
    sd s1, 176(sp)
    sd s2, 184(sp)
    sd s3, 192(sp)
    sd s4, 200(sp)
    sd s5, 208(sp)
    sd s6, 216(sp)
    sd s7, 224(sp)
    sd s8, 232(sp)
    sd s9, 240(sp)
    sd s10, 248(sp)
    sd s11, 256(sp)

    call kernel_trap_handler

    ld t0, 48(sp)
    ld t1, 56(sp)
    ld t2, 64(sp)
    ld t3, 72(sp)
    ld t4, 80(sp)
    ld t5, 88(sp)
    ld t6, 96(sp)

    # load a1..a7
    ld a1, 112(sp)
    ld a2, 120(sp)
    ld a3, 128(sp)
    ld a4, 136(sp)
    ld a5, 144(sp)
    ld a6, 152(sp)
    ld a7, 160(sp)

    # load s0..s11
    ld s0, 168(sp)
    ld s1, 176(sp)
    ld s2, 184(sp)
    ld s3, 192(sp)
    ld s4, 200(sp)
    ld s5, 208(sp)
    ld s6, 216(sp)
    ld s7, 224(sp)
    ld s8, 232(sp)
    ld s9, 240(sp)
    ld s10, 248(sp)
    ld s11, 256(sp)

    addi sp, sp, 264
    sret