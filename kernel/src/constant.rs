use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    VirtAddr,
};

pub const PHYS_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0x0000_4000_0000_0000) };

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub const LOCAL_APIC_ID: u8 = 0;

pub const LOCAL_APIC_TIMER_INIT_COUNT: u32 = u32::MAX;

pub const HPET_INTERVAL: u32 = 10; // 10ms

pub const HEAP_START: u64 = 0x0004_4444_4440 * Size4KiB::SIZE;
pub const HEAP_SIZE: u64 = 128 * 1024 * Size4KiB::SIZE; /* 512 MiB */
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE - 1;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum IOApicInt {
    Timer = IOApicInt::OFFSET,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    Spurious,
    RTC,
    Free9,
    Free10,
    Free11,
    Mouse,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

impl IOApicInt {
    pub const OFFSET: u8 = 32;
}

impl From<IOApicInt> for usize {
    fn from(this: IOApicInt) -> Self {
        this as usize
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LocalApicInt {
    Timer = LocalApicInt::OFFSET,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    Spurious,
    RTC,
    Free9,
    Error,
    // Spec
    Free11,
    Mouse,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

impl LocalApicInt {
    pub const OFFSET: u8 = 128;
}

impl From<LocalApicInt> for usize {
    fn from(this: LocalApicInt) -> Self {
        this as usize
    }
}
