// Uart is a protocol for serial communication.
// It's a way to represent a duplex (two-way) simultaneous communication channel.
// This is specifically a driver for 16550a UARTs, which are commonly used in x86 systems.

// We don't really support so many drivers because we're running this in QEMU.
// We'd need *many many* more drivers to support real hardware.

use core::sync::atomic::Ordering;

use crate::{panic::PANICKED, spinlock::{disable_interrupts, enable_interrupts}};

spinlock!(UART_LOCK);

// The UART like the CLINT is memory-mapped, so we need to know where it is in memory
// QEMU sets the UART to be at 0x10000000
const UART_LOC0: usize = 0x10000000;
const UART_LOC0_IRQ: usize = 10;

// Various "registers" of the UART, note that these are not real registers but rather memory addresses
// When we use memory-mapped I/O, we treat these memory addresses as registers
mod registers {
    /// Receiver holding register
    pub const RHR: usize = 0;
    /// Transmitter holding register
    pub const THR: usize = 0;
    /// Interrupt enable register
    pub const IER: usize = 1;
    /// FIFO control register
    pub const FCR: usize = 2;
    /// Interrupt status register
    pub const ISR: usize = 2;
    /// Line control register
    pub const LCR: usize = 3;
    /// Line status register
    pub const LSR: usize = 5;
}

// This is a helper function to convert a register number to a memory address
// It simply adds the base address of the UART to the register number
#[inline]
const fn reg_map(reg: usize) -> usize {
    UART_LOC0 + reg
}

// Sets the value of a register
// we're using a write_volatile here as we're writing to memory-mapped I/O
// the compiler will by default try to optimize our code and say "oh hey! you're writing to a register multiple times,
// I think we can get rid of that!". The compiler will also try to reorder our writes to the register, which is bad because
// memory-mapped I/O is very sensitive to the order of writes. To prevent this we use write_volatile to tell the compiler
// "hey! don't optimize this!", this is called a "memory fence". For something like C we'd use the `volatile` keyword.
fn write_reg(reg: usize, val: u8) {
    let addr = reg_map(reg);
    unsafe {
        core::ptr::write_volatile(addr as *mut u8, val);
    }
}

// Reads the value of a register
// Same as above, we're using read_volatile to prevent the compiler from optimizing our reads
fn read_reg(reg: usize) -> u8 {
    let addr = reg_map(reg);
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}

// Some constants we'll be setting the UART registers to in a sec, I'll explain them as we go
const LCR_BAUD_LATCH: u8 = 1 << 7;
const LCR_EIGHT_BITS: u8 = 3;
const FCR_FIFO_ENABLE: u8 = 1 << 0;
const FCR_FIFO_CLEAR: u8 = 3 << 1;
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;
const LSR_RX_READY: u8 = 1 << 0;
const LSR_TX_IDLE: u8 = 1 << 5;

pub fn uart_init() {
    // Disable interrupts from the UART
    // This is not the same as system interrupts, but rather the UART's internal interrupts
    // We don't want the UART to interrupt in the middle of us configuring it
    write_reg(registers::IER, 0);

    // Entering to a special mode of the chip that lets us set the baud rate
    write_reg(registers::LCR, LCR_BAUD_LATCH);
    // Set the baud rate to 38,400 this is an agreed timescale for UART communication
    // (The rate at which bits are read over the "wire")
    // We set this by writing to the first two registers of the UART
    // The first register is the least significant byte of the divisor (0x03)
    // The second register is the most significant byte of the divisor (0x00)
    // I'm ngl, I don't fully understand how this sets the baud rate, but it does
    write_reg(0, 0x03);
    write_reg(1, 0x00);

    // Leaving the special mode
    // We're setting the word length to 8 bits here (so we should only send u8s to the UART)
    write_reg(registers::LCR, LCR_EIGHT_BITS);

    // Now we reset and enable the FIFO (First In, First Out) buffer
    // This is a way to store data in a queue-like structure that we can
    // use for reading and writing data to the UART
    write_reg(registers::FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // Finally, we're going to re-enable interrupts for the UART
    // This will let us know when the UART has received a byte or is ready to transmit a byte
    write_reg(registers::IER, IER_RX_ENABLE | IER_TX_ENABLE);
}

pub fn uart_put_c_sync(c: char) {
    // Disable interrupts as we don't want to be interrupted while writing to the UART
    disable_interrupts();

    // If we've panicked we wanna spin here, so we don't lose the panic message
    if PANICKED.load(Ordering::Relaxed) {
        loop {
            core::hint::spin_loop(); // See main.rs for what this does
        }
    }

    while (read_reg(registers::LSR) & LSR_TX_IDLE) == 0 {
        core::hint::spin_loop();
        // Wait for the UART to be ready to transmit
    }

    // Write the character to the UART
    write_reg(registers::THR, c as u8);

    // Re-enable interrupts
    enable_interrupts();
}
