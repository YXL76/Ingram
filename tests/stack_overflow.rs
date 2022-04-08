#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, custom_test_frameworks)]
#![test_runner(ingram::test_runner)]
#![reexport_test_harness_main = "test_main"]

use {
    bootloader::{entry_point, BootInfo},
    ingram::{gdt, println, QEMU_EXIT_HANDLE},
    qemu_exit::QEMUExit,
    spin::Lazy,
    x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

entry_point!(test_kernel_main);

fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    gdt::init();
    IDT.load();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    const ZERO: i32 = 0;
    volatile::Volatile::new(&ZERO).read(); // prevent tail recursion optimizations
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    let double_entry = idt.double_fault.set_handler_fn(test_double_fault_handler);
    unsafe { double_entry.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX) };
    idt
});

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("test tests::stack_overflow ... ok");
    QEMU_EXIT_HANDLE.exit_success();
}
