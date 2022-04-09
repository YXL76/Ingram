mod hole;
mod linked_list_allocator;

use {
    crate::{memory::alloc_virt, println},
    hole::HoleList,
    linked_list_allocator::LockedHeap,
    x86_64::structures::paging::{FrameAllocator, Mapper, PageSize, Size4KiB},
};

pub const HEAP_START: u64 = 0x_4444_4444_0 * Size4KiB::SIZE;
pub const HEAP_SIZE: u64 = 4 * 1024 * Size4KiB::SIZE; /* 16 MiB */
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE - 1;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    alloc_virt(mapper, frame_allocator, HEAP_START, HEAP_END, None);

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as usize, HEAP_SIZE as usize)
    };

    println!("Heap allocated, from {:#x} to {:#x}", HEAP_START, HEAP_END);
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
