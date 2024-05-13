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
            panic!("Interrupts already on");
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

    pub fn init(lock: &mut Option<Spinlock>) {
        *lock = Some(Spinlock::new());
    }

    // Acquire a lock on the Spinlock, this will take care of disabling interrupts
    // and give back a special SpinLockGuard that will enable interrupts when it's dropped
    pub fn acquire(lock: Option<&mut Spinlock>) -> SpinlockGuard {
        disable_interrupts();
        let cpu = Cpu::get_id();
        let lock = lock.expect("Lock not initialized");
        if lock.cpu == Some(cpu) {
            panic!("Lock already acquired by this CPU ({})", cpu);
        }
        while lock.locked.swap(true, Ordering::Acquire) {
            // Spin until we can get the lock
        }
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
        let cpu = Cpu::get_id();
        if self.0.cpu != Some(cpu) {
            panic!("Lock not acquired by this CPU ({})", cpu);
        }
        self.0.cpu = None;
        self.0.locked.store(false, Ordering::Release);
        enable_interrupts();
    }
}

// Macro to define a new spinlock and give it a static lifetime
#[macro_export]
macro_rules! spinlock {
    ($name:ident) => {
        static mut $name: Option<$crate::spinlock::Spinlock> = None;
    };
}
