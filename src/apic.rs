/// https://wiki.osdev.org/APIC
/// https://wiki.osdev.org/APIC_timer
/// https://wiki.osdev.org/HPET
/// https://wiki.osdev.org/PIC
use {
    crate::{
        constant::IO_APIC_OFFSET,
        interrupt::{nmi_disable, nmi_enable},
        memory::{alloc_phys, phys2virt},
        println,
    },
    acpi::{platform::interrupt::Apic, HpetInfo},
    bit_field::BitField,
    core::ptr::{read_volatile, write_volatile},
    raw_cpuid::CpuId,
    x2apic::{
        ioapic::IoApic,
        lapic::{IpiDestMode, LocalApic, LocalApicBuilder, TimerDivide, TimerMode},
    },
    x86_64::{
        instructions::interrupts,
        structures::paging::{FrameAllocator, Mapper, Size4KiB},
    },
};

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    hpet_info: HpetInfo,
    apic: Apic,
) {
    assert!(CpuId::new().get_feature_info().unwrap().has_x2apic());

    interrupts::disable();
    nmi_disable();
    println!("Interrupts disabled");

    /* unsafe {
        asm!("mov al, 0xff
              out 0xa1, al
              out 0x21, al", out("al") _)
    };
    println!("PCI disabled"); */

    let local_apic = {
        let xapic_base = phys2virt(apic.local_apic_address);

        let mut local_apic = LocalApicBuilder::new()
            .timer_vector(32)
            .error_vector(51)
            .spurious_vector(63)
            .timer_mode(TimerMode::Periodic)
            .timer_divide(TimerDivide::Div256)
            .timer_initial(1_000_000_000)
            .ipi_destination_mode(IpiDestMode::Physical)
            .set_xapic_base(xapic_base.as_u64())
            .build()
            .unwrap();

        unsafe { local_apic.enable() };
        println!("Local apic enabled on {:#x}", xapic_base.as_u64());

        local_apic
    };

    apic.io_apics.iter().for_each(|info| {
        let addr = phys2virt(info.address as u64);
        let global_system_interrupt_base = info.global_system_interrupt_base;

        let mut io_apic = unsafe { IoApic::new(addr.as_u64()) };
        // unsafe { io_apic.init((IO_APIC_OFFSET as u32 + global_system_interrupt_base) as u8) };

        /* ioapic.init(irq_offset);

        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_dest(dest); // CPU(s)
        ioapic.set_table_entry(irq_number, entry);

        ioapic.enable_irq(irq_number);*/
    });

    {
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

        let addr = (hpet_info.base_address + 0x10) as *mut u64;
        let mut gen_config = unsafe { read_volatile(addr) };
        gen_config.set_bit(0, true);
        gen_config.set_bit(1, true);
        unsafe { write_volatile(addr, gen_config) }

        println!("HPET enabled",);
    }

    /*
    let irq = hpet::init_hpet_rtc(hpet);
      crate::interrupts::set_handler(irq.map_to_int(0), rtc::rtc_handler);
      kblog!("RTC", "Handler mapped to irq {} via HPET", irq.0);
      hpet::start_hpet(hpet);

      let pit_irq = pit::PIT_IRQ;
      if !pit_irq.has_handler() {
          crate::interrupts::set_handler(pit_irq.map_to_int(0), pit_stub);
      }
      */

    interrupts::enable();
    nmi_enable();
    println!("Interrupts enabled");
}

struct IoApicInfo {
    inner: IoApic,
    id: u8,
    global_system_interrupt_base: u32,
}
