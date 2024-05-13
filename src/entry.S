# We want to setup a stack for each hart

.attribute arch, "rv64gc"
.section .text.entry
.global _entry
_entry:
    la sp, {}
    li a0, 4096
    csrr a1, mhartid
    addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
    call start
spin:
    j spin