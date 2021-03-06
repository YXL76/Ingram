#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, custom_test_frameworks, format_args_nl)]
#![test_runner(ingram_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use {
    core::ptr::read_volatile,
    ingram_kernel::{
        constant::DOUBLE_FAULT_IST_INDEX, entry_point, gdt, println, uart, BootInfo, QEMUExit,
        QEMU_EXIT_HANDLE,
    },
    spin::Lazy,
    x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

entry_point!(test_kernel_main);

fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    uart::init();
    gdt::init();
    IDT.load();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    const ZERO: i32 = 0;
    unsafe { read_volatile(&ZERO) }; // prevent tail recursion optimizations
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    let double_entry = idt.double_fault.set_handler_fn(test_double_fault_handler);
    unsafe { double_entry.set_stack_index(DOUBLE_FAULT_IST_INDEX) };
    idt
});

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("test tests::stack_overflow ... ok");
    QEMU_EXIT_HANDLE.exit_success();
}
