use crate::spinlock;
use crate::spinlock::Spinlock;
use crate::uart::{uart_init, uart_put_c_sync};

spinlock!(CONSOLE_LOCK);

pub fn init_console() {
    unsafe {
        CONSOLE_LOCK = Some(Spinlock::new());
    }
    uart_init();
}

const BACKSPACE: char = '\x08';

pub fn put_c(c: char) {
    if c == BACKSPACE {
        uart_put_c_sync(BACKSPACE);
        uart_put_c_sync(' ');
        uart_put_c_sync(BACKSPACE);
    } else {
        uart_put_c_sync(c);
    }
}
