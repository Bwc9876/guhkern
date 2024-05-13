use crate::consts::NUM_CPUS;

// Represents a CPU core in our system
#[derive(Copy, Clone)]
pub struct Cpu {
    // pub process: Process,
    // pub context: Context,
    pub interrupt_disable_count: usize, // number of times we've disabled interrupts
    pub interrupts_were_on: bool,       // whether or not interrupts were on before we disabled them
}

impl Cpu {
    // WARNING: Must be called with interrupts disabled
    // Get the current ID of this core, this is thanks to use stashing the core ID in the tp register
    pub fn get_id() -> usize {
        unsafe {
            let r: usize;
            core::arch::asm!("mv {}, tp", out(reg) r);
            r
        }
    }

    // WARNING: Must be called with interrupts disabled
    // Will get the current CPU struct for the core that's currently running
    // This is done by getting the core ID and then indexing into the CPUS array (declared below)
    pub fn mine() -> &'static mut Self {
        let id = Self::get_id();
        unsafe { &mut CPUS[id] }
    }
}

// Array of CPUs, one for each core
pub static mut CPUS: [Cpu; NUM_CPUS] = [Cpu {
    // process: Process::new(),
    // context: Context::new(),
    interrupt_disable_count: 0,
    interrupts_were_on: false,
}; 8];
