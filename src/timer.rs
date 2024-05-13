// Handles the timer interrupts

use core::{arch::global_asm, ptr::addr_of_mut};

use riscv::register::{self, stvec::TrapMode};

use crate::consts::NUM_CPUS;

// This function takes care of requesting the timer interrupt
// The timer interrupt is a way for the hardware to tell the CPU to switch contexts.
// We use this as a sort of "heartbeat" for the kernel, we use it to determine when to switch
// what active task we're working on. Handling these interrupts is handled by our scheduler,
// but we need to request the interrupt from the hardware first.
// Unlike other interrupts, these need to be run in machine mode, so we need to request them here
// If our hardware was different (not QEMU), we might be able to do this in supervisor mode
pub fn timer_init() {
    let hart_id = register::mhartid::read();

    // We're going to request a timer from the CLINT (Core Local Interruptor)
    // This is a hardware peripheral that handles interrupts for each core
    // and is used to request timer interrupts per-core.
    const INTERVAL: usize = 1_000_000; // Number of cycles, ~1/10th (100ms) of a second when running with QEMU

    // Instead of using CSRs, we actually directly write to memory to request the timer interrupt
    // This is quite weird so let's break it down:
    // 1. The CLINT is memory-mapped, so we can write to it like any other memory
    // 2. The base address of the CLINT is 0x200_0000, so we do some math to get the address of the MTIMECMP register
    // 3. We need the current time in cycles since boot, which we can get from the mtime register
    // See the constants below this function for more information on exact addresses and calculations
    unsafe {
        // Here we're doing some casting to tell rust we're pointing to a u64
        // `as *const u64` means we're casting the address to a pointer to a u64
        // and `as *mut u64` means we're casting the address to a mutable pointer to a u64
        // So, we cast the address of the MTIMECMP register to a *mut u64
        // And then we set the value at that address to the *const current_time + interval
        // This is how we request the timer interrupt
        *(clint_mtime_cmp_loc(hart_id)) = *(CLINT_MTIME_LOC as *const usize) + INTERVAL;
    }

    // Next we need to prepare something called the MTIME scratch space
    // TIMER_SCRATCH (defined below) is a 2D array that stores some information about the timer interrupt
    // for each core. We need to set the interval, and the address of CLINT_MTIMECMP for each core
    // So, bare with me here
    unsafe {
        // Accessing these static muts is safe as we're only accessing the part
        // of the array that corresponds to the current core, meaning we won't
        // ever access memory that we shouldn't

        // We set 3 and 4 here as we'll use the other slots later for
        // our handler
        TIMER_SCRATCH[hart_id][3] = clint_mtime_cmp_loc(hart_id) as usize;
        TIMER_SCRATCH[hart_id][4] = INTERVAL;

        // Finally, we write the address of the TIMER_SCRATCH to the mscratch register
        // so we can access it later when we handle the interrupt
        register::mscratch::write(addr_of_mut!(TIMER_SCRATCH[hart_id]) as usize);
    }

    // Finally, we're going to point the mtvec register to our timer interrupt handler
    // and then enable the machine timer interrupt
    unsafe {
        // We're setting the mtvec register to point to our timer interrupt handler
        // Direct mode here means we only have one handler for all interrupts
        // If we used vectored mode, we could have multiple handlers for different interrupts but
        // we're going to handle branching in our handler
        register::mtvec::write(timer_entry as usize, TrapMode::Direct);

        // We enable the machine timer interrupt so they actually
        // start happening
        register::mstatus::set_mie(); // Enable machine interrupts
        register::mie::set_mtimer(); // Enable machine timer interrupts specifically
    }
}

static mut TIMER_SCRATCH: [[usize; 5]; NUM_CPUS] = [[0; 5]; NUM_CPUS]; // Scratch space for the timer interrupt

const CLINT_LOC: usize = 0x200_0000; // The base address of the CLINT in memory
const CLINT_MTIME_LOC: usize = CLINT_LOC + 0xBFF8; // The address of the MTIME register

// Calculate the memory location of the MTIMECMP register for a given hart_id
const fn clint_mtime_cmp_loc(hart_id: usize) -> *mut usize {
    (CLINT_LOC + 0x4000 + hart_id * 8) as *mut usize
}

// This asm! block is our timer interrupt handler
// We use it to handle the interrupt and pass a software interrupt to the supervisor
// It's a bit complicated, but I'll break it down in the actual file: timervec.S
// I'd recommend reading *this* file first, as it explains the setup for this handler
global_asm!(include_str!("timervec.S"));

// Expose the timer interrupt entry point to our rust code
extern "C" {
    fn timer_entry();
}
