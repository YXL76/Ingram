#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(ingram::test_runner))]

use {
    bootloader::{entry_point, BootInfo},
    ingram::{hlt_loop, init, println},
};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("Kernel starts");
    // serial_println!("{:#?}", boot_info);

    init(boot_info);

    hlt_loop();
}
