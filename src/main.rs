#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod boot;
mod hw;

use core::panic::PanicInfo;

/// Cate microsecunde ii trebuie procesorului pentru 100.000 de instructiuni goale.
/// Cronometrul merge pe cristal, deci rigla nu se schimba cand schimbam ceasul.
unsafe fn masoara_us() -> u32 {
    let t0 = hw::timer::now_us();
    for _ in 0..100_000 { core::arch::asm!("nop"); }
    let t1 = hw::timer::now_us();
    (t1 - t0) as u32
}

unsafe fn raport() {
    hw::usb_serial::puts("100000 nop = ");
    hw::usb_serial::put_u32(masoara_us());
    hw::usb_serial::puts(" us\r\n");
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    unsafe {
        hw::timer::init();

        // Lasam gazda sa apuce sa deschida portul inainte sa vorbim.
        hw::timer::delay_us(3_000_000);

        hw::usb_serial::puts("\r\n=== INAINTE (pe cristal) ===\r\n");
        raport();
        raport();
        raport();

        hw::clock::cpu_240mhz();

        hw::usb_serial::puts("=== DUPA (pe PLL, 240 MHz) ===\r\n");
        loop {
            raport();
            hw::timer::delay_us(500_000);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
