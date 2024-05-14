// This module provides an abstraction over the UART that we can use to
// print characters to the console. It also provides a function to initialize
// the console which should be called before any other functions in this module.

use crate::spinlock;
use crate::spinlock::Spinlock;
use crate::uart::{uart_init, uart_put_c_sync};

// Here we define a global lock that we will use to synchronize access to the
// console, you'll notice put_c doesn't actually use this lock,
// and that's because put_c is meant for *kernel* code, and therefore
// won't have a lock on it.
spinlock!(CONSOLE_LOCK);

// Here we're simply initializing the console spin lock.
// and then initializing the UART, which is the device we'll be using
// to output text in QEMU.
pub fn init_console() {
    unsafe {
        CONSOLE_LOCK = Some(Spinlock::new());
    }
    uart_init();
}

const BACKSPACE: char = '\x08';

// This is the function we'll use to print characters to the console.
// it can be called from anywhere in the kernel, but we'll most likely use
// println! instead.
pub fn put_c(c: char) {
    if c == BACKSPACE {
        // If our character is a backspace, we'll delete the last character
        // by moving the cursor back one space, printing a space, and then
        // moving the cursor back again, making it look like the character
        // has been deleted.
        uart_put_c_sync(BACKSPACE);
        uart_put_c_sync(' ');
        uart_put_c_sync(BACKSPACE);
    } else {
        uart_put_c_sync(c);
    }
}
