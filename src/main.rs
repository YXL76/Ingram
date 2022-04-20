#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(ingram_kernel::test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

#[macro_use]
extern crate ingram_kernel;
extern crate alloc;

mod port;
mod process;
mod rtc;

use ingram_kernel::{entry_point, BootInfo};

#[cfg(not(test))]
entry_point!(kernel_main);
#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(not(test))]
pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use ingram_kernel::{
        acpi, allocator, apic, constant::PHYS_OFFSET, gdt, interrupt, memory, uart,
    };

    uart::init();

    assert_eq!(
        PHYS_OFFSET.as_u64(),
        boot_info.physical_memory_offset.into_option().unwrap()
    );
    let rsdp_addr = boot_info.rsdp_addr.into_option().unwrap();

    gdt::init();
    interrupt::init();
    let (mut mapper, mut frame_allocator) = unsafe { memory::init(&boot_info.memory_regions) };
    allocator::init(&mut mapper, &mut frame_allocator);
    let (hpet_info, apic, fadt) = acpi::init(rsdp_addr);
    assert_ne!(fadt.century, 0);
    apic::init(&mut mapper, &mut frame_allocator, hpet_info, apic);

    println!("██╗███╗   ██╗ ██████╗ ██████╗  █████╗ ███╗   ███╗");
    println!("██║████╗  ██║██╔════╝ ██╔══██╗██╔══██╗████╗ ████║");
    println!("██║██╔██╗ ██║██║  ███╗██████╔╝███████║██╔████╔██║");
    println!("██║██║╚██╗██║██║   ██║██╔══██╗██╔══██║██║╚██╔╝██║");
    println!("██║██║ ╚████║╚██████╔╝██║  ██║██║  ██║██║ ╚═╝ ██║");
    println!("╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝");

    js_kernel_main(fadt.century)
}

pub fn js_kernel_main(century: u8) -> ! {
    use {
        boa_engine::{
            object::{JsObject, ObjectData, ObjectInitializer},
            property::Attribute,
            Context, JsValue,
        },
        process::KERNEL_MICROTASKS,
    };

    boa_engine::init();
    let mut context = Context::default();

    let mut kernel = ObjectInitializer {
        context: &mut context,
        object: JsObject::from_proto_and_data(None, ObjectData::ordinary()),
    };

    rtc::init(&mut kernel, century);
    port::init(&mut kernel);
    process::init(&mut kernel);
    let kernel = kernel.build();

    context.register_global_property("Kernel", kernel, Attribute::default());
    context.register_global_builtin_function("queueMicrotask", 1, |_this, args, context| {
        let f = args
            .get(0)
            .and_then(|code| code.as_object())
            .ok_or(context.construct_type_error("missing func"))?
            .clone();

        unsafe { KERNEL_MICROTASKS.get_unchecked() }
            .push(f)
            .unwrap();

        Ok(JsValue::undefined())
    });

    if let Err(err) = context.eval(include_bytes!("../dist/index.js")) {
        panic!("{}", err.to_string(&mut context).unwrap());
    }

    while let Some(f) = unsafe { KERNEL_MICROTASKS.get_unchecked() }.pop() {
        let _ = f.call(&JsValue::null(), &[], &mut context).unwrap();
    }

    ingram_kernel::hlt_loop();
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
