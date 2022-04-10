/// https://wiki.osdev.org/Interrupts
/// https://wiki.osdev.org/NMI
use {
    crate::{
        apic::LOCAL_APIC,
        constant::{
            DOUBLE_FAULT_IST_INDEX, LOCAL_APIC_ERROR, LOCAL_APIC_SPURIOUS, LOCAL_APIC_TIMER,
        },
        println,
    },
    spin::Lazy,
    x86_64::{
        instructions::port::Port,
        set_general_handler,
        structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
    },
};

pub fn init() {
    IDT.load();
    println!("IDT loaded at {:p}", &IDT);
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    fn my_general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
        todo!(
            "handle irq {} {:?} code: {:?}",
            index,
            stack_frame,
            error_code
        )
    }

    // set all entries
    set_general_handler!(&mut idt, my_general_handler);

    idt.breakpoint.set_handler_fn(breakpoint_handler);

    let double_entry = idt.double_fault.set_handler_fn(double_fault_handler);
    unsafe { double_entry.set_stack_index(DOUBLE_FAULT_IST_INDEX) };

    idt.page_fault.set_handler_fn(page_fault_handler);

    idt[36].set_handler_fn(io_apic_com1_handler);

    idt[LOCAL_APIC_TIMER.into()].set_handler_fn(apic_timer_handler);
    idt[LOCAL_APIC_ERROR.into()].set_handler_fn(apic_timer_handler);
    idt[LOCAL_APIC_SPURIOUS.into()].set_handler_fn(apic_timer_handler);

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

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let pl =
        x86_64::registers::segmentation::SegmentSelector(stack_frame.code_segment as u16).rpl();

    println!(
        "page fault: frame {:?}, error code: {:?} pl: {:?}",
        stack_frame, error_code, pl
    );
}

pub extern "x86-interrupt" fn io_apic_com1_handler(_stack_frame: InterruptStackFrame) {
    crate::print!(".");
    unsafe { LOCAL_APIC.get().unwrap().lock().end_of_interrupt() };
}

pub extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {
    crate::print!(".");
    unsafe { LOCAL_APIC.get().unwrap().lock().end_of_interrupt() };
}

pub fn nmi_enable() {
    let mut nmi_port = Port::<u8>::new(0x70);
    let val = unsafe { nmi_port.read() };
    unsafe { nmi_port.write(val & 0x7F) };
}

pub fn nmi_disable() {
    let mut nmi_port = Port::<u8>::new(0x70);
    let val = unsafe { nmi_port.read() };
    unsafe { nmi_port.write(val | 0x80) };
}
