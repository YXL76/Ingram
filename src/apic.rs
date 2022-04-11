/// https://wiki.osdev.org/APIC
/// https://wiki.osdev.org/APIC_timer
/// https://wiki.osdev.org/HPET
/// https://wiki.osdev.org/IOAPIC
/// https://wiki.osdev.org/PIC
/// https://wiki.osdev.org/Timer_Interrupt_Sources
use {
    crate::{
        constant::{
            IOApicInt, LocalApicInt, HPET_INTERVAL, LOCAL_APIC_ID, LOCAL_APIC_TIMER_INIT_COUNT,
        },
        interrupt::{nmi_disable, nmi_enable},
        memory::alloc_phys,
        println,
    },
    acpi::{platform::interrupt::Apic, HpetInfo},
    alloc::collections::BTreeMap,
    bit_field::BitField,
    core::ptr::{read_volatile, write_volatile},
    spin::Once,
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

    unsafe { (&mut *LOCAL_APIC.as_mut_ptr()).enable() };
    nmi_enable();
    interrupts::enable();
    // wait for [crete::interrupt::io_apic_timer_handler]
    x86_64::instructions::hlt();
    println!("Local apic enabled");
    println!("Interrupts enabled");
}

pub static LOCAL_APIC: Once<LocalApic> = Once::new();

fn init_local_apic() {
    // Do not provide xapic base address, it is only used in xapic.
    // We assume the machine support x2apic

    // let xapic_base = phys2virt(apic.local_apic_address);
    let local_apic = LocalApicBuilder::new()
        .timer_vector(LocalApicInt::Timer.into())
        .error_vector(LocalApicInt::Error.into())
        .spurious_vector(LocalApicInt::Spurious.into())
        .timer_mode(TimerMode::Periodic)
        .timer_divide(TimerDivide::Div256)
        .timer_initial(LOCAL_APIC_TIMER_INIT_COUNT)
        .ipi_destination_mode(IpiDestMode::Physical)
        // .set_xapic_base(xapic_base.as_u64())
        .build()
        .unwrap();

    assert!(unsafe { local_apic.is_bsp() });

    LOCAL_APIC.call_once(move || local_apic);
}

pub static IO_APICS: Once<IoApics> = Once::new();

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
        unsafe { io_apic.init(IOApicInt::OFFSET) };

        for i in 0..=max_entry {
            unsafe { io_apic.disable_irq(i) };
        }

        let mut io_apics = IoApics {
            irq_mappings,
            io_apic,
            gsi_base,
            max_entry,
        };

        io_apics.enable_irq(IOApicInt::Timer);
        println!("IRQ {:#?} enabled", IOApicInt::Timer);
        io_apics.enable_irq(IOApicInt::COM1);
        println!("IRQ {:#?} enabled", IOApicInt::COM1);

        io_apics
    });

    println!("I/O apics initialized");
}

pub struct IoApics {
    irq_mappings: BTreeMap<u8, u8>,
    io_apic: IoApic,
    gsi_base: u8,
    max_entry: u8,
}

impl IoApics {
    fn enable_irq(&mut self, irq: IOApicInt) {
        let gsi = self.find_io_apic(&irq);

        let mut entry = unsafe { self.io_apic.table_entry(gsi) };
        // let mut entry = RedirectionTableEntry::default();
        // entry.set_mode(IrqMode::Fixed);
        // entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_vector(irq as u8);
        entry.set_dest(LOCAL_APIC_ID);
        unsafe { self.io_apic.set_table_entry(gsi, entry) };

        unsafe { self.io_apic.enable_irq(gsi) };
    }

    pub fn disable_irq(&mut self, irq: IOApicInt) {
        let gsi = self.find_io_apic(&irq);

        unsafe { self.io_apic.disable_irq(gsi) };
    }

    fn find_io_apic(&mut self, irq: &IOApicInt) -> u8 {
        let irq = *irq as u8 - IOApicInt::OFFSET;
        let gsi = *self.irq_mappings.get(&irq).unwrap_or(&irq);
        assert!(gsi >= self.gsi_base && gsi <= self.gsi_base + self.max_entry);
        gsi
    }
}

/// Use HPET to figure Local APIC Timer freq
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

    let period = gen_caps.get_bits(32..=63);
    let freq = 10u64.pow(15) / period;
    let ticks = freq / 1000; // ticks for 1ms

    // Timer 0
    let cmp_addr = (hpet_info.base_address + 0x108 + 0x20 * 0) as *mut u64;
    unsafe { write_volatile(cmp_addr, ticks * HPET_INTERVAL as u64) }
    // Main Counter
    let cmp_addr = (hpet_info.base_address + 0x0F0) as *mut u64;
    unsafe { write_volatile(cmp_addr, 0) }

    /* let nums = gen_caps.get_bits(8..=15);
    for i in 0..nums {
        let addr = addr + 0x100 + 0x20 * i;
        let n_caps = unsafe { read_volatile(addr as *const u64) };
        assert!(n_caps.get_bit(5));
    } */

    // Enable HPET
    let addr = (hpet_info.base_address + 0x10) as *mut u64;
    let mut gen_config = unsafe { read_volatile(addr) };
    gen_config.set_bit(0, true); // enable "legacy replacement" mapping
    gen_config.set_bit(1, true); // enable timer interrupts
    unsafe { write_volatile(addr, gen_config) }

    // Config timer 0
    let cfg_addr = (hpet_info.base_address + 0x100 + 0x20 * 0) as *mut u64;
    let mut cfg = unsafe { read_volatile(cfg_addr) };
    cfg.set_bit(2, true); // enable triggering of interrupts
    cfg.set_bit(3, false); // enable non-periodic mode
    unsafe { write_volatile(cfg_addr, cfg) }
}
