#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use uefi::{prelude::*, ResultExt};

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect_success("Failed to initialize utils");

    // reset console before doing anything else
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset output buffer");

    // Print out UEFI revision number
    {
        let rev = system_table.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());

        info!("UEFI {}.{}", major, minor);
    }

    memory_map(&system_table.boot_services());

    Status::SUCCESS
}

fn memory_map(bt: &BootServices) {
    const EFI_PAGE_SIZE: u64 = 0x1000;

    // Get the estimated map size
    let map_size = bt.memory_map_size();

    // Build a buffer bigger enough to handle the memory map
    let mut buffer = vec![0; map_size << 1];

    let (_k, desc_iter) = bt
        .memory_map(&mut buffer)
        .expect_success("Failed to retrieve UEFI memory map");

    desc_iter.for_each(|desc| {
        let size = desc.page_count * EFI_PAGE_SIZE;
        info!(
            "{:<2} {:#11x} - {:#11x} ({:<10} KiB)",
            desc.ty.0,
            desc.phys_start,
            desc.phys_start + size,
            size
        );
    });
}
