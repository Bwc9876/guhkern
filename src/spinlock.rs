// This file contains common code to take and release a spinlock. This is a simple way to handle
// mutual exclusion in a multi-core system. The idea is that a core will try to take the lock, and
// if it can't, it will keep trying until it can. This is a very simple way to handle mutual

use core::{
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

use riscv::register;

use crate::cpu::Cpu;

// First, get whether or not interrupts are enabled so we can set it back to that once the lock is released
// Then, disable interrupts so we can safely take the lock without being interrupted
pub fn disable_interrupts() {
    unsafe {
        let old = register::sstatus::read().sie();
        register::sstatus::clear_sie();
        let cpu = Cpu::mine();
        if cpu.interrupt_disable_count == 0 {
            cpu.interrupts_were_on = old;
        }
        cpu.interrupt_disable_count += 1;
    }
}

// Once the lock is released, we need to set the interrupt state back to what it was before we took the lock
pub fn enable_interrupts() {
    unsafe {
        // Check if interrupts are on, if they are, panic as we're enabling interrupts while holding a lock
        let is_on = register::sstatus::read().sie();
        if is_on {
            panic!("interrupts_on_enable");
        }
        // Get the current CPU, we know interrupts are off so we can call this.
        let cpu = Cpu::mine();
        // Decrement the interrupt disable count
        cpu.interrupt_disable_count -= 1;
        // If the count is 0 and interrupts were on before we disabled them, set the SIE bit again
        // as we're at the same state we were before we took the lock, and interrupts were enabled in
        // that state
        if cpu.interrupt_disable_count == 0 && cpu.interrupts_were_on {
            register::sstatus::set_sie();
        }
    }
}

pub struct Spinlock {
    pub cpu: Option<usize>,
    pub locked: AtomicBool,
}

impl Spinlock {
    // Constructor for a new Spinlock
    // This creates a spin::Mutex with a Spinlock inside, and returns it
    pub const fn new() -> Spinlock {
        Spinlock {
            cpu: None,
            locked: AtomicBool::new(false),
        }
    }

    // Initialize a Spinlock, this will set the lock to Some(Spinlock)
    // We want to make sure only one thread initialized a spinlock
    // which is why we don't just set it directly in the static and instead
    // make an option
    pub fn init(lock: &mut Option<Spinlock>) {
        *lock = Some(Spinlock::new());
    }

    // Acquire a lock on the Spinlock, this will take care of disabling interrupts
    // and give back a special SpinLockGuard that will enable interrupts and unlock when it's dropped
    // This function will panic if the lock is already acquired by the current CPU, or if the lock is not initialized
    pub fn acquire(lock: Option<&mut Spinlock>) -> SpinlockGuard {
        // Disable interrupts as we really really don't want to be interrupted while taking a lock
        disable_interrupts();
        // Get out current CPU to make sure the lock isn't being held by our CPU already
        let cpu = Cpu::get_id();
        let lock = lock.expect("lock_uninit");
        if lock.cpu == Some(cpu) {
            panic!("lock_acq_same_hart");
        }
        // Keep spinning until we can get the lock, this is a very simple way to handle mutual exclusion
        while lock.locked.swap(true, Ordering::Acquire) {
            // Spin until we can get the lock
        }
        // We now have the lock! Set the CPU to the current CPU and return a SpinlockGuard
        lock.cpu = Some(cpu);
        SpinlockGuard(lock)
    }
}

// This is a guard for the Spinlock, it will enable interrupts when it's dropped
pub struct SpinlockGuard<'a>(&'a mut Spinlock);

// Implement Deref for SpinlockGuard so we can access the Spinlock inside easily
impl Deref for SpinlockGuard<'_> {
    type Target = Spinlock;

    fn deref(&self) -> &Spinlock {
        self.0
    }
}

// Implement Drop for SpinlockGuard so we can unlock and enable interrupts when it's dropped
impl Drop for SpinlockGuard<'_> {
    fn drop(&mut self) {
        // Get the CPU ID and ensure we're the one that acquired the lock
        // If we're not, panic as something went wrong
        let cpu = Cpu::get_id();
        if self.0.cpu != Some(cpu) {
            panic!("lock_rel_diff_hart");
        }
        // We know we have the lock, so we can release it, set the CPU to None
        // and then re-enable interrupts so other CPUs can take the lock
        self.0.cpu = None;
        self.0.locked.store(false, Ordering::Release);
        enable_interrupts();
    }
}

// Macro to define a new spinlock and give it a static lifetime
// This starts it out as None, and it should be initialized later
#[macro_export]
macro_rules! spinlock {
    ($name:ident) => {
        static mut $name: Option<$crate::spinlock::Spinlock> = None;
    };
}
