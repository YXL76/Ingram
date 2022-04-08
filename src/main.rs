#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    const_mut_refs,
    custom_test_frameworks,
    default_alloc_error_handler
)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod allocator;
mod gdt;
mod interrupts;
mod memory;
mod uart;

use {
    bootloader::{entry_point, BootInfo},
    core::panic::PanicInfo,
    qemu_exit::QEMUExit,
    x86_64::VirtAddr,
};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("Kernel starts");
    // serial_println!("{:#?}", boot_info);

    let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);

    gdt::init();
    interrupts::init(boot_info.rsdp_addr.into_option().unwrap());

    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::GlobalFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init(&mut mapper, &mut frame_allocator);

    #[cfg(test)]
    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}

pub const QEMU_EXIT_HANDLE: qemu_exit::X86 = qemu_exit::X86::new(0xf4, 0x21);

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("running {} tests", tests.len());
    for test in tests {
        test.run();
    }
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
        print!("{} ... ", core::any::type_name::<T>());
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
