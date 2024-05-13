# Handles the timer interrupt
# I recommend you read timer.rs first to understand how the timer interrupt is set up
# This code is called when the timer interrupt is triggered

.section .text.timervec
.globl timer_entry
timer_entry:
    csrrw a0, mscratch, a0 # Save the scratch register 
    sd a1, 0(a0) # Save the argument to the scratch register; TIMER_SCRATCH[hart_id][0]
    sd a2, 8(a0) # ...and TIMER_SCRATCH[hart_id][1]
    sd a3, 16(a0) # ...and TIMER_SCRATCH[hart_id][2]
    ld a1, 24(a0) # This is CLINT_MTIMECMP that we set before; TIMER_SCRATCH[hart_id][3]
    ld a2, 32(a0) # This is the interval that we set before; TIMER_SCRATCH[hart_id][4]
    ld a3, 0(a1) # Loading the current value of the MTIMECMP register
    add a3, a3, a2 # Adding the interval to the current value
    sd a3, 0(a1) # Writing the new value back to the MTIMECMP register, this is how we request the next interrupt
    li a1, 2 # Arrange the arguments for the supervisor software interrupt
    csrw sip, a1 # Setting the supervisor interrupt pending register to request the supervisor software interrupt
    ld a3, 16(a0) # Load back TIMER_SCRATCH[hart_id][2]
    ld a2, 8(a0) # ...and TIMER_SCRATCH[hart_id][1]
    ld a1, 0(a0) # ...and TIMER_SCRATCH[hart_id][0]
    csrrw a0, mscratch, a0 # Restore the scratch register
    mret # Return from the interrupt
