//! This project is meant to be a simple kernel that runs on the RISC-V architecture.
//! This is mostly a learning project for me, but im going to leave comments *everywhere* so that
//! I can make sense of it later.

// We're not using the standard library, so we need to tell the compiler that
// This is because the std lib relies on OS-specific implementations, which we don't have
// Instead, rust provides the `core` package, which is all the parts of std that don't rely on OS-specific implementations
#![no_std]
// We also don't want the default entrypoint, because we're going to define our own
// We won't have access to the C runtime or Rust runtime in bare-metal, so we need to define our own entrypoint
// that uses neither
#![no_main]
// Enable the `start` attribute, which allows us to define our own entrypoint (see start::start)
#![feature(start)]
#![allow(dead_code)]

use core::{
    arch::global_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use consts::NUM_CPUS;
use cpu::Cpu;

// Module for interacting with the console
mod console;

mod consts;

// Module for managing the current core
mod cpu;

// Defining our panic handler in this module
mod panic;

// Module for handling println! and print! macros
#[macro_use]
mod println;

// Module for handling mutually exclusive spin locks
#[macro_use]
mod spinlock;

// Actual entrypoint (bootstrapping code) is in this module
mod start;

// This defines the setup and handling of machine timer interrupts
mod timer;

// Module for handling UART communication
mod uart;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
// When this function is called, we're in supervisor mode and our kernel is bootstrapped
// See start.rs for the actual bootstrapping code
pub fn main() -> ! {
    let cpu_id = Cpu::get_id();
    if cpu_id == 0 {
        console::init_console();
        println::init_println();
        println!("Guhkern booting!");

        INITIALIZED.store(true, Ordering::SeqCst);
    } else {
        while !INITIALIZED.load(Ordering::SeqCst) {
            // Wait for CPU 0 to finish initializing
            core::hint::spin_loop();
        }
        println!("CPU {} starting", cpu_id);
    }
    loop {
        core::hint::spin_loop();
    }
}

static STACK0: [u8; 4096 * NUM_CPUS] = [0; 4096 * NUM_CPUS];

global_asm!(include_str!("entry.S"), sym STACK0);
