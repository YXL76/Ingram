#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

mod display;
mod error;
mod fs;
mod io;

use {
    alloc::vec::Vec,
    core::{mem, slice},
    error::Result,
    fs::File,
    goblin::elf::{program_header::PT_LOAD, Elf, ProgramHeaders},
    io::Read,
    uefi::{
        proto::media::file::FileMode,
        table::{
            boot::{AllocateType, BootServices, MemoryType},
            Boot, SystemTable,
        },
        Handle, Status,
    },
    x86::{
        bits64::paging::BASE_PAGE_SIZE,
        controlregs::{cr0, cr4, Cr0, Cr4},
        cpuid::CpuId,
    },
};

#[uefi::prelude::entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).expect("Failed to initialize utils");
    let bs = st.boot_services();
    // Print out UEFI revision number
    {
        let rev = st.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());
        info!("UEFI v{}.{}", major, minor);
    }

    display::init(bs);
    check();

    let mut buf = Vec::new();
    let (entry, program_headers) = parse_kernel(bs, &mut buf).unwrap();

    load_kernel(bs, &mut buf, &program_headers);
    let _entry: EntryPoint = unsafe { mem::transmute(entry) };
    clear(&mut st);

    Status::SUCCESS
}

type EntryPoint = fn(u64) -> !;

/// Check CPU features
fn check() {
    let cpuid = CpuId::new();
    let vendor = cpuid.get_vendor_info().expect("Failed to get vendor info");
    info!("{} CPU", vendor.as_str());
    let ef = cpuid
        .get_extended_processor_and_feature_identifiers()
        .expect("Failed to get extended processor and feature identifiers");
    assert!(ef.has_64bit_mode());

    let cr0 = unsafe { cr0() };
    assert!(cr0.contains(Cr0::CR0_PROTECTED_MODE));
    assert!(cr0.contains(Cr0::CR0_ENABLE_PAGING));

    let cr4 = unsafe { cr4() };
    assert!(cr4.contains(Cr4::CR4_ENABLE_PAE));
    // assert!(cr4.contains(Cr4::CR4_ENABLE_PSE));
}

/// Reset console
fn clear(st: &mut SystemTable<Boot>) {
    st.stdout()
        .reset(false)
        .expect("Failed to reset output buffer");
}

fn parse_kernel<'a>(bs: &'a BootServices, buf: &'a mut Vec<u8>) -> Result<(u64, ProgramHeaders)> {
    let mut file = File::open(bs, r"EFI\Boot\kernel", FileMode::Read)?;
    file.read_to_end(buf)?;
    let elf = Elf::parse(buf).unwrap();
    assert!(elf.is_64);
    Ok((elf.entry, elf.program_headers))
}

fn load_kernel(bs: &BootServices, buf: &mut [u8], phs: &ProgramHeaders) {
    let loadables = phs
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD)
        .collect::<Vec<_>>();

    let start = loadables.iter().min_by_key(|ph| ph.p_vaddr).unwrap();
    let end = loadables.iter().max_by_key(|ph| ph.p_vaddr).unwrap();
    info!("Kernel: {:#x} ~ {:#x}", start.p_vaddr, end.p_vaddr);

    for ph in loadables {
        const LOWER: usize = BASE_PAGE_SIZE - 1;

        let vaddr = ph.p_vaddr as usize;
        let mensz = ph.p_memsz as usize;

        let dest = vaddr & !LOWER;
        let page_num = ((mensz + vaddr & LOWER) >> 12) + 1;

        bs.allocate_pages(
            AllocateType::Address(dest),
            MemoryType::LOADER_CODE,
            page_num,
        )
        .expect("Failed to allocate memory");

        unsafe { bs.set_mem(dest as *mut u8, page_num * BASE_PAGE_SIZE, 0) };
        let mem = unsafe { slice::from_raw_parts_mut(vaddr as *mut u8, mensz) };
        let elf = &mut buf[ph.file_range()];
        if mem.len() == elf.len() {
            mem.copy_from_slice(elf);
        }
    }
}
