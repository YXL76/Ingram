/// https://wiki.osdev.org/ACPI
use {
    crate::{memory::phys2virt, println},
    acpi::{
        platform::{interrupt::Apic, ProcessorInfo},
        AcpiHandler, AcpiTables, HpetInfo, InterruptModel, PciConfigRegions, PlatformInfo,
    },
    core::ptr::NonNull,
};

pub fn init(rsdp_addr: u64) -> (HpetInfo, Apic) {
    let acpi_tables = unsafe { AcpiTables::from_rsdp(AcpiHdl, rsdp_addr as usize) }.unwrap();

    let pci_config_regions = PciConfigRegions::new(&acpi_tables).unwrap();
    println!("PCI: {:?}", pci_config_regions);

    let PlatformInfo {
        power_profile,
        interrupt_model,
        processor_info,
        pm_timer,
    } = acpi_tables.platform_info().unwrap();

    println!("Power profile: {:?}", power_profile);

    let ProcessorInfo {
        boot_processor: processor,
        application_processors: app_processor,
    } = processor_info.unwrap();
    assert!(app_processor.is_empty(), "Do not support multi-core");
    println!("Processor: {:?}", processor);

    println!("PM timer: {:?}", pm_timer.unwrap());

    (
        HpetInfo::new(&acpi_tables).unwrap(),
        match interrupt_model {
            InterruptModel::Apic(apic) => apic,
            _ => panic!("apic not supported"),
        },
    )
}

#[derive(Clone)]
struct AcpiHdl;

impl AcpiHandler for AcpiHdl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(phys2virt(physical_address as u64).as_u64() as *mut _),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // noop
    }
}
