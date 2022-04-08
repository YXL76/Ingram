use {
    spin::Lazy,
    x86_64::{
        instructions::{
            segmentation::{Segment, CS},
            tables::load_tss,
        },
        structures::{
            gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
            tss::TaskStateSegment,
        },
        VirtAddr,
    },
};

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code);
        load_tss(GDT.1.tss);
    }
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        let stack_end = stack_start + STACK_SIZE;
        stack_end
    };
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors { code, tss })
});

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
}
