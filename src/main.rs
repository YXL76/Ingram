#![no_std]
#![no_main]
#![feature(format_args_nl)]
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
    use {
        boa_engine::Context,
        ingram_kernel::{hlt_loop, init},
    };

    init(boot_info);

    println!("Hello World!");
    let mut context = Context::default();
    println!(
        "{}",
        context
            .eval("'Hello World from Javascript!'")
            .unwrap()
            .to_string(&mut context)
            .unwrap()
    );

    hlt_loop();
}

#[cfg(test)]
pub fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    use ingram_kernel::{uart, QEMUExit, QEMU_EXIT_HANDLE};
    uart::init();
    print!("");
    test_main();
    QEMU_EXIT_HANDLE.exit_success()
}

/// See https://github.com/rust-lang/libm/issues/258
/// Copies from https://github.com/rust-lang/compiler-builtins/blob/19d53ba6d86fe64b89f28dc8dba02eeffb15c8f8/src/math.rs
pub mod fmin {
    #[no_mangle]
    pub extern "C" fn fmin(x: f64, y: f64) -> f64 {
        libm::fmin(x, y)
    }
}

/// See https://github.com/rust-lang/libm/issues/258
/// Copies from https://github.com/rust-lang/compiler-builtins/blob/19d53ba6d86fe64b89f28dc8dba02eeffb15c8f8/src/math.rs
pub mod fmax {
    #[no_mangle]
    pub extern "C" fn fmax(x: f64, y: f64) -> f64 {
        libm::fmax(x, y)
    }
}

/// See https://github.com/rust-lang/libm/issues/258
/// Copies from https://github.com/rust-lang/compiler-builtins/blob/19d53ba6d86fe64b89f28dc8dba02eeffb15c8f8/src/math.rs
pub mod fmod {
    #[no_mangle]
    pub extern "C" fn fmod(x: f64, y: f64) -> f64 {
        libm::fmod(x, y)
    }
}
