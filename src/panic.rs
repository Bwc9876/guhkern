use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;

use crate::println;
use crate::println::PRINTLN_LOCK;

pub static PANICKED: AtomicBool = AtomicBool::new(false);

// Halt on panic, don't allow us to return
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        PRINTLN_LOCK.0 = false;
        println!("Kernel panicked!");
        println!("Reason: {}", _info);
        PANICKED.store(true, core::sync::atomic::Ordering::SeqCst);
    }
    loop {}
}
