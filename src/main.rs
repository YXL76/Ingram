#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use {
    bootloader::{entry_point, BootInfo},
    core::panic::PanicInfo,
    qemu_exit::QEMUExit,
};

mod interrupts;
mod serial;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    interrupts::init_idt();

    #[cfg(test)]
    test_main();

    loop {}
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    QEMU_EXIT_HANDLE.exit_success();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}

const QEMU_EXIT_HANDLE: qemu_exit::X86 = qemu_exit::X86::new(0xf4, 0x21);

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    QEMU_EXIT_HANDLE.exit_failure();
}
