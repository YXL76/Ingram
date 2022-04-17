use {spin::Once, uart_16550::SerialPort, x86_64::instructions::interrupts::without_interrupts};

pub static SERIAL1: Once<SerialPort> = Once::new();

pub fn init() {
    SERIAL1.call_once(|| {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        serial_port
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    without_interrupts(|| {
        unsafe { &mut *SERIAL1.as_mut_ptr() }
            .write_fmt(args)
            .unwrap()
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::uart::_print(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::uart::_print(format_args_nl!($($arg)*));
    })
}
