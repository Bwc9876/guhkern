// This module defines how we can print to the console
// using format strings and arguments

use core::fmt::{self, Write};

use crate::{console, spinlock::Spinlock};

pub struct PrintlnLock(pub bool, pub Option<Spinlock>);

// Have a lock here to prevent multiple cores from printing at the same time
// Here's an example of what could happen if we don't have a lock:
// Highlight from when locks weren't working:
// - CCPPUU  12  ssttaarrttiinngg
// - CCPU PU 21  stsatratrting i
//   ng
// Highlight from when locks were working:
// - CPU 1 starting
//   CPU 2 starting
pub static mut PRINTLN_LOCK: PrintlnLock = PrintlnLock(false, None);

// Initialized out locks and sets the PRINTLN_LOCK.0 to true
// this means we'll require the lock to print
pub fn init_println() {
    unsafe {
        Spinlock::init(&mut PRINTLN_LOCK.1);
        PRINTLN_LOCK.0 = true;
    }
}

// Unit struct to implement the Write trait
struct Output;

// This implementation of Write will allow us to use `Write::write_fmt` function
// in order to output fmt::Arguments to the console
impl core::fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            console::put_c(c);
        }
        Ok(())
    }
}

// Base print function, this will lock if we're supposed to and
// then write the arguments to the console
pub fn print(args: fmt::Arguments) {
    let _lock = if unsafe { PRINTLN_LOCK.0 } {
        Some(unsafe { Spinlock::acquire(PRINTLN_LOCK.1.as_mut()) })
    } else {
        None
    };
    let mut out = Output;
    out.write_fmt(args).unwrap();
    drop(_lock);
}

// Println function, this will call the print function with a newline
pub fn println(args: fmt::Arguments) {
    print(format_args!("{}\n", args));
}

// Macros to allow us to use the print and println functions
// we have output now!
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::println::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::println::println(format_args!($($arg)*));
    };
}
