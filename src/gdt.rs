use {
    crate::{constant::DOUBLE_FAULT_IST_INDEX, println},
    spin::Lazy,
    x86_64::{
        instructions::{
            segmentation::{Segment, CS, DS, ES, SS},
            tables::load_tss,
        },
        structures::{
            gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
            paging::{PageSize, Size4KiB},
            tss::TaskStateSegment,
        },
        VirtAddr,
    },
};

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code);
        SS::set_reg(GDT.1.data);
        DS::set_reg(GDT.1.data);
        ES::set_reg(GDT.1.data);
        load_tss(GDT.1.tss);
    }

    println!("GDT loaded at {:p}", &GDT.0)
}

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    const STACK_SIZE: usize = (Size4KiB::SIZE * 5) as usize;

    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
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
    let data = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors { code, data, tss })
});

struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    tss: SegmentSelector,
}
