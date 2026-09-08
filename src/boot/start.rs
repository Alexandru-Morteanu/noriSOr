use core::arch::global_asm;

global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",
    "  movi a1, _stack_top",
    "  j    _start_rust",
);

extern "C" {
    static mut _bss_start: u32;
    static mut _bss_end: u32;
}

#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    let mut p = core::ptr::addr_of_mut!(_bss_start);
    let end = core::ptr::addr_of_mut!(_bss_end);
    while p < end {
        p.write_volatile(0);
        p = p.add(1);
    }
    crate::rust_main()
}
