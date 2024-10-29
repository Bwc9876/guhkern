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
#![feature(asm_const)]
#![allow(dead_code)]

// Needed to use Vec and String, since we have a GlobalAllocator setup in [kalloc.rs] we can use it
#[macro_use]
extern crate alloc;

use core::{
    arch::global_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::string::{String, ToString};
use consts::NUM_CPUS;
use cpu::Cpu;
use vm::kvm_init_hart;

// Module for interacting with the console
mod console;

// Constants used throughout the kernel
mod consts;

// Module for managing the current core
mod cpu;

// Module for handling memory allocation in user space
mod kalloc;

// Defining our panic handler in this module
mod panic;

// Module for handling println! and print! macros
#[macro_use]
mod println;

mod plic;

// Module for handling mutually exclusive spin locks
#[macro_use]
mod spinlock;

// Actual entrypoint (bootstrapping code) is in this module
mod start;

// This defines the setup and handling of machine timer interrupts
mod timer;

// Module for handling UART communication
mod uart;

// Module for interacting with a virtual disk
mod virtio;

// Module for handling Virtual Memory and Page Tables
mod vm;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
// Even though this is called main, this isn't actually the start of our program!
// When we get here the kernel has already been loaded into memory and the CPU has been initialized
// Look below this function to start the long journey of the kernel bootstrapping process
pub fn main() -> ! {
    // Ok! So we're in the main function and we're officially started!
    // First thing we need to do is get the ID of the CPU we're running on,
    // this is so we can only have one CPU do initialization of shared resources such
    // as the console and the println! macros
    let cpu_id = Cpu::get_id();
    if cpu_id == 0 {
        // If we're the first CPU, we need to initialize our shared resources
        console::init_console();
        println::init_println();
        // First output to the console! If we get here we're doing good because we can now debug
        // *much* easier
        println!("Kernel booting!");
        kalloc::kinit();
        vm::kvm_init_base();
        vm::kvm_init_hart();
        println!("KVM Init");

        println!("CPU 0 Finished Setup!");
        // Signal to the other CPUs that we're done initializing
        // This will allow the other CPUs to start
        INITIALIZED.store(true, Ordering::SeqCst);
    } else {
        // If we're not CPU 0, we're going to be waiting on the sidelines until
        // CPU 0 finishes initializing, telling us we can start
        while !INITIALIZED.load(Ordering::SeqCst) {
            // Wait for CPU 0 to finish initializing
            // This hint is a special way for the compiler to know we're busy-waiting (spin-locking)
            // It emits a special instruction that signals to the CPU that we're waiting for something
            // and the CPU can then so some optimizations to make this more efficient
            core::hint::spin_loop();
        }
        // CPU 0 is done and we have access to shared resources using locks
        println!("CPU {} starting", cpu_id);
        kvm_init_hart();
    }
    // TEMP: Just spin forever for now, we'd want to head into our scheduler from here
    loop {
        core::hint::spin_loop();
    }
}

// === START HERE ===

// So first things first as we're booting up the kernel, we need to define the entrypoint
// that everything will start from. This is the start of the bootstrapping process.
// To do this we need to write some assembly, I put this in a separate file called entry.S
// that you should go look at the second you see include_str!("entry.S") below

// First we need to initialize the stack for each CPU
// This is going to be a 4KB stack for each CPU
// The reason we do this is we don't want CPUs to share a stack
// because then they'll get in each other's way
// So we create a new slice of 4KB for each CPU

const STACK_PER_CPU: usize = 4096;

static STACK0: [u8; STACK_PER_CPU * NUM_CPUS] = [0; STACK_PER_CPU * NUM_CPUS];

// Here we're telling Rust to include the assembly code from entry.S
// I could have wrote it all in here, but it's easier to follow with syntax highlighting
// The sym STACK0 is a symbol (hence the `sym` keyword) that we're telling Rust to insert into the assembly code
// This symbol is the address of the stack we just defined
// This will get inserted where the {} is in the assembly code
// Also, we insert the constant STACK_PER_CPU into the assembly code so
// we have a single place where we can change the stack size

global_asm!(include_str!("entry.S"), sym STACK0, const STACK_PER_CPU);
