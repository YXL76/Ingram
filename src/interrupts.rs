use {
    crate::{gdt, println},
    // acpi::AcpiTables,
    spin::Lazy,
    x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

pub fn init(_rsdp_addr: u64) {
    IDT.load();
    // unsafe { AcpiTables::from_rsdp(rsdp_addr as usize) }.unwrap();
    // instructions::interrupts::enable();
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    let double_entry = idt.double_fault.set_handler_fn(double_fault_handler);
    unsafe { double_entry.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX) };
    idt
});

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
