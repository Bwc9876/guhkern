// This module defines special behaviour when our kernel panics
// This is a special case where we don't want to return from the panic handler

use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;

use crate::println;
use crate::println::PRINTLN_LOCK;

// This static acts as a signal to the rest of the kernel that a panic has occurred
// This is used to prevent the kernel from continuing to output messages to the console
pub static PANICKED: AtomicBool = AtomicBool::new(false);

// Halt on panic, don't allow us to return
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        // First we set the PRINTLN_LOCK to false, meaning
        // println will no longer attempt to acquire the lock
        // this is so if a panic occurs in println! we don't
        // infinitely loop as we try to acquire the lock
        PRINTLN_LOCK.0 = false;
        // Now we print the panic message to the screen, this won't
        // lock as we've set the lock to false
        println!("!=!=!=! Kernel panicked! !=!=!=!");
        println!("Reason: {}", info);
        // Finally we set the PANICKED flag to true, this will
        // prevent other cores from continuing to output messages
        PANICKED.store(true, core::sync::atomic::Ordering::SeqCst);
    }
    loop {}
}
