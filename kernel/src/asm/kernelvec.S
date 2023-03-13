.section .text
.global timervec
timervec:
    # now a0 points to scratch[mhartid]
    csrrw a0, mscratch, a0
    # save registers
    sd a1, 0(a0)
    sd a2, 8(a0)
    sd a3, 16(a0)

    # rewrite a1,a3,a4 to prepare next timer interrup
    ld a1, 24(a0) # MTIMECMP addr
    ld a2, 32(a0) # write timer interval to a2
    ld a3, 0(a1)  #
    add a3, a3, a2
    sd a3, 0(a1)

    # trigger a supervisor interrupt
    li a1, 2
    csrw sip, a1
   
    ld a3, 16(a0)
    ld a2, 8(a0)
    ld a1, 0(a0)
    csrrw a0, mscratch, a0
    mret