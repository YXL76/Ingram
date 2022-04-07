use {
    crate::serial_println,
    spin::Once,
    x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init_idt() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    });
    idt.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[cfg(test)]
mod tests {
    use x86_64::instructions;

    #[test_case]
    fn test_breakpoint_exception() {
        instructions::interrupts::int3();
    }
}
