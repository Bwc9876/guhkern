# This is the start of our boot code, everything will begin here.
# First thing we want to do is get out of assembly and into Rust as soon as possible.
# To do this we are going to setup a stack for each hart (CPU core) and then call into Rust.

# In order for QEMU to actually see this assembly code we need to make sure it's at position 0x80000000
# This is because the second QEMU is done initializing it jumps to this address.
# To do this we need a custom linker script (linker.ld) that ensures this code is at the right address.

# The .attribute arch is needed to specify to Rust that we can use `mul`
.attribute arch, "rv64gc" 
.section .text.entry
.global _entry
_entry:
    # Here we're loading the stack pointer to the base of our entire stack 
    # (the actual stack symbol will be inserted in the braces by Rust)
    # The stack pointer defines where our thread's stack starts in memory.
    # We're not done yet though, as this would mean the stack would be shared between all harts.
    la sp, {}
    # Here we're simply loading the size of each hart's stack into a0
    # This is also inserted by Rust at the braces
    li a0, {}
    # Now we're going to get the hartid (the id of the current CPU core) and store it in a1
    csrr a1, mhartid
    # Now we add 1 to the hartid (it's 0 indexed) and multiply it by the stack size 
    # to get where the given hart's stack should start relative to the 
    # base of the entire stack
    addi a1, a1, 1
    mul a0, a0, a1
    # Finally we add the offset to the base of the entire stack to get the
    # actual stack pointer for the current hart and store it in sp
    add sp, sp, a0
    # Now we're ready to call into Rust! See start.rs for the next steps.
    call start
# This section represents a spin loop that will be used to keep the hart running
# in the event that `call start` returns. This is mostly a safety net to ensure
# that the hart doesn't keep executing random instructions after the Rust code
spin:
    j spin
