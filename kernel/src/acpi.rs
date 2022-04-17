/// https://wiki.osdev.org/ACPI
use {
    crate::{constant::LOCAL_APIC_ID, memory::phys2virt, println},
    acpi::{
        fadt::Fadt,
        platform::{interrupt::Apic, ProcessorInfo},
        sdt::Signature,
        AcpiHandler, AcpiTables, HpetInfo, InterruptModel, PciConfigRegions, PhysicalMapping,
        PlatformInfo,
    },
    core::ptr::NonNull,
    x86_64::structures::paging::{PageSize, Size4KiB},
};

pub fn init(rsdp_addr: u64) -> (HpetInfo, Apic, PhysicalMapping<AcpiHdl, Fadt>) {
    let acpi_tables = unsafe { AcpiTables::from_rsdp(AcpiHdl, rsdp_addr as usize) }.unwrap();

    let fadt = unsafe { acpi_tables.get_sdt::<Fadt>(Signature::FADT) }
        .unwrap()
        .unwrap();

    let pci_config_regions = PciConfigRegions::new(&acpi_tables).unwrap();
    println!("PCI: {:?}", pci_config_regions);

    let PlatformInfo {
        power_profile,
        interrupt_model,
        processor_info,
        pm_timer: _,
    } = acpi_tables.platform_info().unwrap();

    println!("Power profile: {:?}", power_profile);

    let ProcessorInfo {
        boot_processor: processor,
        application_processors: app_processor,
    } = processor_info.unwrap();
    assert_eq!(processor.local_apic_id, LOCAL_APIC_ID as u32);
    assert!(app_processor.is_empty(), "Do not support multi-core");

    (
        HpetInfo::new(&acpi_tables).unwrap(),
        match interrupt_model {
            InterruptModel::Apic(apic) => apic,
            _ => panic!("apic not supported"),
        },
        fadt,
    )
}

#[derive(Clone)]
pub struct AcpiHdl;

impl AcpiHandler for AcpiHdl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let page_size = Size4KiB::SIZE as usize;
        // address maybe not aligned, so we align it manually
        let aligned_start = physical_address & !(page_size - 1);
        let aligned_end = (physical_address + size + page_size - 1) & !(page_size - 1);

        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(phys2virt(physical_address as u64).as_u64() as *mut _),
            size,
            aligned_end - aligned_start,
            Self,
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // noop
    }
}
