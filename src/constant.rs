use x86_64::VirtAddr;

pub const PHYS_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0x0000_4000_0000_0000) };

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub const IO_APIC_OFFSET: u8 = 128;
