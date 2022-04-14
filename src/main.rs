#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(ingram_kernel::test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

#[macro_use]
extern crate ingram_kernel;

use ingram_kernel::{entry_point, BootInfo};

#[cfg(not(test))]
entry_point!(kernel_main);
#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(not(test))]
pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use ingram_kernel::{hlt_loop, init};

    init(boot_info);

    println!("Hello World!");

    hlt_loop();
}

#[cfg(test)]
pub fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    use ingram_kernel::{uart, QEMUExit, QEMU_EXIT_HANDLE};
    uart::init();
    test_main();
    QEMU_EXIT_HANDLE.exit_success()
}
