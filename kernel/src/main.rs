#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn _start(_info_addr: usize) -> ! {
    loop {}
}
