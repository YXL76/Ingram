mod hole;
mod linked_list_allocator;

use {
    hole::HoleList,
    linked_list_allocator::LockedHeap,
    x86_64::{
        structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
        VirtAddr,
    },
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator) }
            .unwrap()
            .flush();
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
#[inline]
fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

#[cfg(test)]
mod tests {
    use {
        super::HEAP_SIZE,
        alloc::{boxed::Box, vec::Vec},
    };

    #[test_case]
    fn test_simple_allocation() {
        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);
        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }

    #[test_case]
    fn large_vec() {
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    }

    #[test_case]
    fn test_many_boxes() {
        for i in 0..HEAP_SIZE {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }
}
