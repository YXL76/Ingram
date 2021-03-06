#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    const_mut_refs,
    format_args_nl,
    type_alias_impl_trait
)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

pub mod acpi;
pub mod allocator;
pub mod apic;
pub mod constant;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod uart;

pub use {
    bootloader::{entry_point, BootInfo},
    qemu_exit::QEMUExit,
};

use core::panic::PanicInfo;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn default_handler(layout: core::alloc::Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

/* ---------- Testing ---------- */

pub const QEMU_EXIT_HANDLE: qemu_exit::X86 = qemu_exit::X86::new(0xf4, 0x21);

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    println!();
    println!("test result: ok.");
    QEMU_EXIT_HANDLE.exit_success();
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("test {} ... ", core::any::type_name::<T>());
        self();
        println!("ok");
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    QEMU_EXIT_HANDLE.exit_failure();
}
