#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod boot;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
