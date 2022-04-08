#![no_std]
#![feature(abi_x86_interrupt, alloc_error_handler, const_mut_refs)]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

mod allocator;
pub mod gdt;
mod interrupts;
mod memory;
pub mod uart;

use {bootloader::BootInfo, core::panic::PanicInfo, qemu_exit::QEMUExit, x86_64::VirtAddr};

pub fn init(boot_info: &'static mut BootInfo) {
    let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);

    gdt::init();
    interrupts::init(boot_info.rsdp_addr.into_option().unwrap());

    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::GlobalFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init(&mut mapper, &mut frame_allocator);
}

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

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

pub const QEMU_EXIT_HANDLE: qemu_exit::X86 = qemu_exit::X86::new(0xf4, 0x21);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);
    test_main();
    QEMU_EXIT_HANDLE.exit_failure();
}

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
