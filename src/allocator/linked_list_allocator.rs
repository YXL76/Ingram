// Copyright (c) 2016 Philipp Oppermann
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use {
    super::HoleList,
    core::{
        alloc::{GlobalAlloc, Layout},
        ops::Deref,
        ptr::NonNull,
    },
    spin::Mutex,
};

/// A fixed size heap backed by a linked list of free memory blocks.
pub struct Heap {
    bottom: usize,
    size: usize,
    used: usize,
    holes: HoleList,
}

impl Heap {
    const fn empty() -> Heap {
        Heap {
            bottom: 0,
            size: 0,
            used: 0,
            holes: HoleList::empty(),
        }
    }

    /// Initializes an empty heap
    ///
    /// # Safety
    ///
    /// This function must be called at most once and must only be used on an
    /// empty heap.
    pub unsafe fn init(&mut self, heap_bottom: usize, heap_size: usize) {
        self.bottom = heap_bottom;
        self.size = heap_size;
        self.used = 0;
        self.holes = HoleList::new(heap_bottom, heap_size);
    }

    /* /// Initialize an empty heap with provided memory.
    ///
    /// The caller is responsible for procuring a region of raw memory that may be utilized by the
    /// allocator. This might be done via any method such as (unsafely) taking a region from the
    /// program's memory, from a mutable static, or by allocating and leaking such memory from
    /// another allocator.
    ///
    /// The latter method may be especially useful if the underlying allocator does not perform
    /// deallocation (e.g. a simple bump allocator). Then the overlaid linked-list-allocator can
    /// provide memory reclamation.
    ///
    /// # Panics
    ///
    /// This method panics if the heap is already initialized.
    pub fn init_from_slice(&mut self, mem: &'static mut [MaybeUninit<u8>]) {
        assert_eq!(self.bottom, 0, "The heap has already been initialized.");
        let size = mem.len();
        let address = mem.as_ptr() as usize;
        // SAFETY: All initialization requires the bottom address to be valid, which implies it
        // must not be 0. Initially the address is 0. The assertion above ensures that no
        // initialization had been called before.
        // The given address and size is valid according to the safety invariants of the mutable
        // reference handed to us by the caller.
        unsafe { self.init(address, size) }
    } */

    /* /// Creates a new heap with the given `bottom` and `size`. The bottom address must be valid
    /// and the memory in the `[heap_bottom, heap_bottom + heap_size)` range must not be used for
    /// anything else. This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_bottom: usize, heap_size: usize) -> Heap {
        if heap_size < HoleList::min_size() {
            Self::empty()
        } else {
            Heap {
                bottom: heap_bottom,
                size: heap_size,
                used: 0,
                holes: HoleList::new(heap_bottom, heap_size),
            }
        }
    } */

    /* /// Creates a new heap from a slice of raw memory.
    ///
    /// This has the same effect as [`init_from_slice`] on an empty heap, but it is combined into a
    /// single operation that can not panic.
    pub fn from_slice(mem: &'static mut [MaybeUninit<u8>]) -> Heap {
        let size = mem.len();
        let address = mem.as_ptr() as usize;
        // SAFETY: The given address and size is valid according to the safety invariants of the
        // mutable reference handed to us by the caller.
        unsafe { Self::new(address, size) }
    } */

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `None`.
    /// This function scans the list of free memory blocks and uses the first block that is big
    /// enough. The runtime is in O(n) where n is the number of free blocks, but it should be
    /// reasonably fast for small allocations.
    pub fn allocate_first_fit(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        match self.holes.allocate_first_fit(layout) {
            Ok((ptr, aligned_layout)) => {
                self.used += aligned_layout.size();
                Ok(ptr)
            }
            Err(err) => Err(err),
        }
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate_first_fit` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// This function walks the list of free memory blocks and inserts the freed block at the
    /// correct place. If the freed block is adjacent to another free block, the blocks are merged
    /// again. This operation is in `O(n)` since the list needs to be sorted by address.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.used -= self.holes.deallocate(ptr, layout).size();
    }

    /* /// Returns the bottom address of the heap.
    pub fn bottom(&self) -> usize {
        self.bottom
    } */

    /* /// Returns the size of the heap.
    pub fn size(&self) -> usize {
        self.size
    } */

    /* /// Return the top address of the heap
    pub fn top(&self) -> usize {
        self.bottom + self.size
    } */

    /* /// Returns the size of the used part of the heap
    pub fn used(&self) -> usize {
        self.used
    } */

    /* /// Returns the size of the free part of the heap
    pub fn free(&self) -> usize {
        self.size - self.used
    } */

    /* /// Extends the size of the heap by creating a new hole at the end
    ///
    /// # Safety
    ///
    /// The new extended area must be valid
    pub unsafe fn extend(&mut self, by: usize) {
        let top = self.top();
        let layout = Layout::from_size_align(by, 1).unwrap();
        self.holes
            .deallocate(NonNull::new_unchecked(top as *mut u8), layout);
        self.size += by;
    } */
}

pub struct LockedHeap(Mutex<Heap>);

impl LockedHeap {
    pub const fn empty() -> LockedHeap {
        LockedHeap(Mutex::new(Heap::empty()))
    }

    /* /// Creates a new heap with the given `bottom` and `size`. The bottom address must be valid
    /// and the memory in the `[heap_bottom, heap_bottom + heap_size)` range must not be used for
    /// anything else. This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_bottom: usize, heap_size: usize) -> LockedHeap {
        LockedHeap(Mutex::new(Heap {
            bottom: heap_bottom,
            size: heap_size,
            used: 0,
            holes: HoleList::new(heap_bottom, heap_size),
        }))
    } */
}

impl Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}
