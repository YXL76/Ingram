use x86_64::VirtAddr;

pub const PHYS_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0x0000_4000_0000_0000) };

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub const LOCAL_APIC_ID: u8 = 0;

pub const INT_OFFSET: u8 = 32;

const LOCAL_APIC_INT_OFFSET: u8 = 128;

pub const LOCAL_APIC_TIMER: u8 = LOCAL_APIC_INT_OFFSET;

pub const LOCAL_APIC_ERROR: u8 = LOCAL_APIC_INT_OFFSET + 1;

pub const LOCAL_APIC_SPURIOUS: u8 = LOCAL_APIC_INT_OFFSET + 2;
