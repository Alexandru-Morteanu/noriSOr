#![allow(dead_code)]

pub mod wdt;
pub mod usb_serial;
pub mod timer;
pub mod clock;

pub const RTC_CNTL: usize = 0x6000_8000;
pub const UART0:    usize = 0x6000_0000;
pub const SYSTEM:   usize = 0x600C_0000;
pub const USB_DEV:  usize = 0x6003_8000;
pub const TIMG0:    usize = 0x6001_F000;

#[inline(always)]
pub unsafe fn wr(addr: usize, val: u32) { (addr as *mut u32).write_volatile(val); }

#[inline(always)]
pub unsafe fn rd(addr: usize) -> u32 { (addr as *const u32).read_volatile() }
