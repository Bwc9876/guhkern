pub const NUM_CPUS: usize = 8; // Max number of CPUs in our system
pub const KERNEL_START: usize = 0x8000_0000; // Start of kernel memory
pub const PHYS_STOP: usize = KERNEL_START + 128 * 1024 * 1024;
