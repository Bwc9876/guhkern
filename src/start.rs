use riscv::register::{self, mstatus::MPP, satp::Mode};

#[no_mangle]
// Start entrypoint, this is the first bit of Rust ever run in our kernel
// we're coming from entry.S here, which is the true entrypoint of the kernel.
// In this function we configure the CPU to run how we want it to, and then jump to our `main` function
// where the real kernel logic starts (I promise this time we're actually starting the kernel after this)
pub fn start() {
    // Set our previous privilege mode to supervisor
    // Also set our exception program counter to `main`, which is where we want to jump to when we `mret`
    // This is a surprise tool that will help us later
    // (We're going to do some initialization that can only be done in machine mode in a sec,
    // but we're going to go "back" to supervisor mode when we finish and actually jump to `main`)
    unsafe {
        // We want our kernel to run in supervisor mode because running machine mode
        // Gives us access to many ways to harm the system, supervisor allows us to still
        // have some privileges but prevents us from doing anything too dangerous
        register::mstatus::set_mpp(MPP::Supervisor);
        // Here we're casting `main` to a usize, which is creating a raw pointer to the function
        // When the CPU jumps to it (we call mret), it will be in supervisor mode and our kernel will officially start
        register::mepc::write(crate::main as usize);
    }

    // Disabling paging for right now, we'll enable it later when we're in the kernel
    // This means all addresses are physical addresses, which is what we want for now
    // as we're not doing any memory management yet.
    // Most of the time this is implicitly set to 0, but we're doing it explicitly here for clarity
    unsafe { register::satp::set(Mode::Bare, 0, 0) }

    // Here we're delegating all traps (exceptions and interrupts) to supervisor mode
    // This is because we're going to be running in supervisor mode, and we want to handle all traps there
    // This is a safety measure, as we don't want to handle traps in machine mode
    unsafe {
        // I'm doing a manual asm! here because the riscv crate seemingly doesn't have a way to set all bits
        // of the medeleg register at once, so I'd need to call each one individually which is a pain
        // So to breakdown the line below:
        // 1. `csrrw` is an assembly instruction that writes a value to a CSR (medeleg in this case)
        // 2. `x0` is the register we're writing to, which is a "scratch" register that always contains 0
        // 3. `0x302` is the CSR number for medeleg
        // 4. `0xffff` is the value we're writing to medeleg, which is all bits set to 1
        core::arch::asm!("csrrw x0, 0x302, {}", in(reg) 0xffff);

        // Same thing as above, but for mideleg
        // mideleg is the machine interrupt delegation register, which is similar to medeleg but for interrupts
        core::arch::asm!("csrrw x0, 0x303, {}", in(reg) 0xffff);

        // Finally we want to enable supervisor interrupts, which will allow us to handle interrupts in supervisor mode
        // We want to enable three:
        // 1. Supervisor software interrupts
        // 2. Supervisor timer interrupts
        // 3. Supervisor external interrupts
        // We're basically advertising that we're ready to handle these interrupts to the CPU
        register::sie::set_ssoft();
        register::sie::set_stimer();
        register::sie::set_sext();
    }

    // Now we want to do some physical memory protection
    // RISC-V allows us to protect memory by setting the PMP (Physical Memory Protection) registers
    // We're going to set them to allow all access to all memory as we're in supervisor mode,
    // but we'll come back to this later when we're doing memory management
    // We set this to usize::MAX because we want to allow all access to all memory,
    // and the size of usize represents all addresses in memory
    register::pmpaddr0::write(usize::MAX);
    // We also set the PMP configuration register to 0xf, which allows all access to all memory
    register::pmpcfg0::write(0xf);

    // Here we're going to initialize the timer, which we'll use to handle time-based interrupts
    // See the function's comments for more information
    crate::timer::timer_init();

    // One last thing before we can get the kernel running,
    // We need to grab the `hartid` (hardware thread ID) from the mhartid CSR,
    // which is a unique identifier for each hardware thread in a multicore system.
    // The reason we need this is to write it to the `tp` register, which is the thread pointer register
    // We perform this transfer because `mhartid` is only able to be read from in Machine mode,
    // So we need to grab it here and stash it in `tp` so we can access it later (Cpu::get_id)
    let hart_id = register::mhartid::read();
    // TP is a general purpose register so we can just write to it normally
    unsafe {
        core::arch::asm!("mv tp, {}", in(reg) hart_id);
    }

    // And that's it! We're ready to jump to `main` and start running our kernel!!
    // We do this by calling the `mret` instruction, which will jump to the address in `mepc`.
    // And since we set MPP to supervisor and set the epc to `main`, we'll be in supervisor mode
    unsafe {
        core::arch::asm!("mret"); // Exciting!
    }
}
