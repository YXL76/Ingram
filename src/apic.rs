/// https://wiki.osdev.org/APIC
/// https://wiki.osdev.org/APIC_timer
/// https://wiki.osdev.org/HPET
/// https://wiki.osdev.org/IOAPIC
/// https://wiki.osdev.org/PIC
use {
    crate::{
        constant::{
            INT_OFFSET, LOCAL_APIC_ERROR, LOCAL_APIC_ID, LOCAL_APIC_SPURIOUS, LOCAL_APIC_TIMER,
        },
        interrupt::{nmi_disable, nmi_enable},
        memory::alloc_phys,
        println,
    },
    acpi::{platform::interrupt::Apic, HpetInfo},
    alloc::collections::BTreeMap,
    bit_field::BitField,
    core::ptr::{read_volatile, write_volatile},
    spin::{Mutex, Once},
    x2apic::{
        ioapic::IoApic,
        lapic::{IpiDestMode, LocalApic, LocalApicBuilder, TimerDivide, TimerMode},
    },
    x86_64::{
        instructions::{interrupts, port::PortWriteOnly},
        structures::paging::{FrameAllocator, Mapper, Size4KiB},
    },
};

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    hpet_info: HpetInfo,
    apic: Apic,
) {
    interrupts::disable();
    nmi_disable();
    println!("Interrupts disabled");

    unsafe { PortWriteOnly::new(0xa1).write(u8::MAX) };
    unsafe { PortWriteOnly::new(0x21).write(u8::MAX) };
    println!("PCI disabled");

    init_local_apic();
    init_io_apics(mapper, frame_allocator, &apic);
    init_hpet(mapper, frame_allocator, &hpet_info);

    interrupts::enable();
    nmi_enable();
    println!("Interrupts enabled");
}

pub static LOCAL_APIC: Once<Mutex<LocalApic>> = Once::new();

fn init_local_apic() {
    // Do not provide xapic base address, it is only used in xapic.
    // We assume the machine support x2apic

    // let xapic_base = phys2virt(apic.local_apic_address);

    LOCAL_APIC.call_once(|| {
        let mut local_apic = LocalApicBuilder::new()
            .timer_vector(LOCAL_APIC_TIMER.into())
            .error_vector(LOCAL_APIC_ERROR.into())
            .spurious_vector(LOCAL_APIC_SPURIOUS.into())
            .timer_mode(TimerMode::OneShot)
            .timer_divide(TimerDivide::Div256)
            .timer_initial(100_000_000)
            .ipi_destination_mode(IpiDestMode::Physical)
            // .set_xapic_base(xapic_base.as_u64())
            .build()
            .unwrap();

        unsafe { local_apic.enable() };
        assert!(unsafe { local_apic.is_bsp() });
        unsafe { local_apic.disable_timer() }

        Mutex::new(local_apic)
    });

    println!("Local apic enabled");
}

static IO_APICS: Once<Mutex<IoApics>> = Once::new();

fn init_io_apics(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    apic: &Apic,
) {
    IO_APICS.call_once(|| {
        let mut irq_mappings = BTreeMap::new();
        for i in &apic.interrupt_source_overrides {
            // TODO: Save `polarity` and `trigger_mode`
            irq_mappings.insert(i.isa_source, i.global_system_interrupt as u8);
        }

        // Only use the first one
        let addr = apic.io_apics[0].address as u64;
        let gsi_base = apic.io_apics[0].global_system_interrupt_base as u8;
        alloc_phys(mapper, frame_allocator, addr, addr, None);

        let mut io_apic = unsafe { IoApic::new(addr) };
        let max_entry = unsafe { io_apic.max_table_entry() };
        unsafe { io_apic.init(INT_OFFSET) };

        for i in 0..=max_entry {
            unsafe { io_apic.disable_irq(i) };
        }

        let mut io_apics = IoApics {
            irq_mappings,
            io_apic,
            gsi_base,
            max_entry,
        };

        io_apics.enable_irq(4);

        Mutex::new(io_apics)
    });

    println!("I/O apics init");
}

struct IoApics {
    irq_mappings: BTreeMap<u8, u8>,
    io_apic: IoApic,
    gsi_base: u8,
    max_entry: u8,
}

impl IoApics {
    fn enable_irq(&mut self, irq: u8) {
        let gsi = self.find_io_apic(&irq);

        let mut entry = unsafe { self.io_apic.table_entry(gsi) };
        // let mut entry = RedirectionTableEntry::default();
        // entry.set_mode(IrqMode::Fixed);
        // entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_vector(gsi + INT_OFFSET);
        entry.set_dest(LOCAL_APIC_ID);
        unsafe { self.io_apic.set_table_entry(gsi, entry) };

        unsafe { self.io_apic.enable_irq(gsi) };
    }

    fn find_io_apic(&mut self, irq: &u8) -> u8 {
        let gsi = *self.irq_mappings.get(irq).unwrap_or(irq);
        assert!(gsi >= self.gsi_base && gsi <= self.gsi_base + self.max_entry);
        gsi
    }
}

fn init_hpet(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    hpet_info: &HpetInfo,
) {
    let addr = hpet_info.base_address as u64;
    alloc_phys(mapper, frame_allocator, addr, addr, None);

    let gen_caps = unsafe { read_volatile(addr as *const u64) };
    assert!(gen_caps.get_bit(13));
    println!(
        "Found HPET at {:x}, rev. id: {:x}, vendor id: {:x}",
        addr,
        gen_caps.get_bits(0..=7),
        gen_caps.get_bits(16..=31)
    );

    /* let nums = gen_caps.get_bits(8..=15);
    for i in 0..nums {
        let addr = addr + 0x100 + 0x20 * i;
        let n_caps = unsafe { read_volatile(addr as *const u64) };
        assert!(n_caps.get_bit(5));
    } */

    let period = gen_caps.get_bits(32..=63);
    let freq = 10u64.pow(15) / period;

    // Start
    let addr = (hpet_info.base_address + 0x10) as *mut u64;
    let mut gen_config = unsafe { read_volatile(addr) };
    gen_config.set_bit(0, true);
    gen_config.set_bit(1, true);
    unsafe { write_volatile(addr, gen_config) }

    println!("HPET enabled",);
}
