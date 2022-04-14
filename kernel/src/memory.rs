use {
    crate::constant::PHYS_OFFSET,
    alloc::collections::VecDeque,
    bootloader::boot_info::{MemoryRegionKind, MemoryRegions},
    x86_64::{
        registers::control::Cr3,
        structures::paging::{
            FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageSize, PageTable,
            PageTableFlags, PhysFrame, Size4KiB,
        },
        PhysAddr, VirtAddr,
    },
};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(
    memory_regions: &'static MemoryRegions,
) -> (OffsetPageTable<'static>, GlobalFrameAllocator) {
    let level_4_table = active_level_4_table();
    (
        OffsetPageTable::new(level_4_table, PHYS_OFFSET),
        GlobalFrameAllocator::init(memory_regions),
    )
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table() -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys2virt(phys.as_u64());
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub fn phys2virt(addr: u64) -> VirtAddr {
    PHYS_OFFSET + addr
}

pub fn alloc_virt(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    start: u64,
    end: u64,
    flags: Option<PageTableFlags>,
) {
    let page_range = {
        let start = VirtAddr::new(start);
        let end = VirtAddr::new(end);
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT | flags.unwrap_or(PageTableFlags::WRITABLE);
        unsafe { mapper.map_to(page, frame, flags, frame_allocator) }
            .unwrap()
            .flush();
    }
}

pub fn alloc_phys(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    start: u64,
    end: u64,
    flags: Option<PageTableFlags>,
) {
    let frame_range = {
        let start = PhysAddr::new(start);
        let end = PhysAddr::new(end);
        let start_frame = PhysFrame::containing_address(start);
        let end_frame = PhysFrame::containing_address(end);
        PhysFrame::range_inclusive(start_frame, end_frame)
    };

    for frame in frame_range {
        let flags = PageTableFlags::PRESENT
            | flags.unwrap_or(PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE);
        unsafe { mapper.identity_map(frame, flags, frame_allocator) }
            .unwrap()
            .flush();
    }
}

type UsableFrameIterator = impl Iterator<Item = PhysFrame>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct GlobalFrameAllocator {
    iter: UsableFrameIterator,
    deallocated: Option<VecDeque<PhysFrame>>,
}

impl GlobalFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        // get usable regions from memory map
        let regions = memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| (r.start)..(r.end));
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(Size4KiB::SIZE as usize));
        // create `PhysFrame` types from the start addresses
        let iter = frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));

        GlobalFrameAllocator {
            iter,
            deallocated: None,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.deallocated
            .as_mut()
            .and_then(|deallocated| deallocated.pop_front())
            .or(self.iter.next())
    }
}

impl FrameDeallocator<Size4KiB> for GlobalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let deallocated = self.deallocated.get_or_insert(VecDeque::new());
        deallocated.push_back(frame);
    }
}
