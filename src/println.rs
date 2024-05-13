use core::fmt::{self, Write};

use crate::{console, spinlock::Spinlock};

pub struct PrintlnLock(pub bool, pub Option<Spinlock>);

pub static mut PRINTLN_LOCK: PrintlnLock = PrintlnLock(false, None);

pub fn init_println() {
    unsafe {
        Spinlock::init(&mut PRINTLN_LOCK.1);
        PRINTLN_LOCK.0 = true;
    }
}

struct Output;

impl core::fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            console::put_c(c);
        }
        Ok(())
    }
}

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

pub fn println(args: fmt::Arguments) {
    print(format_args!("{}\n", args));
}

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
