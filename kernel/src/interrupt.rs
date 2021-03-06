/// https://wiki.osdev.org/Interrupts
/// https://wiki.osdev.org/NMI
use {
    crate::{
        apic::LOCAL_APIC,
        constant::{IOApicInt, LocalApicInt, DOUBLE_FAULT_IST_INDEX},
        println,
        uart::SERIAL1,
    },
    spin::Lazy,
    x86_64::{
        instructions::{
            port::Port,
            segmentation::{Segment, DS},
        },
        set_general_handler,
        structures::{
            gdt::SegmentSelector,
            idt::{InterruptDescriptorTable, InterruptStackFrame},
        },
    },
};

pub fn init() {
    IDT.load();
    println!("IDT loaded at {:p}", &IDT);
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    fn my_general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
        todo!("irq {} {:?} code: {:?}", index, stack_frame, error_code)
    }

    // set all entries
    set_general_handler!(&mut idt, my_general_handler);

    idt.breakpoint.set_handler_fn(breakpoint_handler);

    let double_entry = idt.double_fault.set_handler_fn(double_fault_handler);
    unsafe { double_entry.set_stack_index(DOUBLE_FAULT_IST_INDEX) };

    idt[IOApicInt::COM1.into()].set_handler_fn(io_apic_com1_handler);

    idt[LocalApicInt::Timer.into()].set_handler_fn(local_apic_timer_handler);

    idt
});

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn io_apic_com1_handler(_stack_frame: InterruptStackFrame) {
    use core::fmt::Write;

    let serial1 = unsafe { &mut *SERIAL1.as_mut_ptr() };
    let ch = serial1.receive() as char;
    serial1.write_char(ch).unwrap();
    unsafe { (&mut *LOCAL_APIC.as_mut_ptr()).end_of_interrupt() };
}

extern "x86-interrupt" fn local_apic_timer_handler(_stack_frame: InterruptStackFrame) {
    unsafe { DS::set_reg(SegmentSelector(1)) };
    unsafe { (&mut *LOCAL_APIC.as_mut_ptr()).end_of_interrupt() };
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
