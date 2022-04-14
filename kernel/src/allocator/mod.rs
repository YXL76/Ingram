mod hole;
mod linked_list_allocator;

use {
    crate::{
        constant::{HEAP_END, HEAP_SIZE, HEAP_START},
        memory::alloc_virt,
        println,
    },
    hole::HoleList,
    linked_list_allocator::LockedHeap,
    x86_64::structures::paging::{FrameAllocator, Mapper, Size4KiB},
};

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
