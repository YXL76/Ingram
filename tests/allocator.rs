#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ingram_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use {
    alloc::{boxed::Box, vec::Vec},
    ingram_kernel::{
        allocator, constant::HEAP_SIZE, entry_point, gdt, interrupt, memory, uart, BootInfo,
        QEMUExit, QEMU_EXIT_HANDLE,
    },
};

entry_point!(test_kernel_main);

fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    uart::init();
    gdt::init();
    interrupt::init();
    let (mut mapper, mut frame_allocator) = unsafe { memory::init(&boot_info.memory_regions) };
    allocator::init(&mut mapper, &mut frame_allocator);

    {
        // test_simple_allocation
        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);
        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }
    {
        // large_vec
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    }
    {
        // test_many_boxes
        for i in 0..HEAP_SIZE {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }

    QEMU_EXIT_HANDLE.exit_success()
}
