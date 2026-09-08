use core::arch::global_asm;

global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",

    // 1. Fereastra de registre, în stare cunoscută.
    "  movi a0, 0",
    "  wsr.windowbase a0",
    "  movi a0, 1",
    "  wsr.windowstart a0",
    "  rsync",

    // 2. Starea procesorului: pornește ferestrele, ieși din modul de excepție.
    //    Bitul 18 = ferestre active. Bitul 5 = mod utilizator.
    "  movi a0, 0x00040020",
    "  wsr.ps a0",
    "  rsync",

    // 3. Fără adresă de întoarcere — nu ne mai întoarcem nicăieri.
    "  movi a0, 0",

    // 4. Stiva.
    "  movi a1, _stack_top",

    // 5. În Rust.
    "  j    _start_rust",
);

extern "C" {
    static mut _bss_start: u32;
    static mut _bss_end: u32;
}

#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    crate::hw::wdt::disable();

    let mut p = core::ptr::addr_of_mut!(_bss_start);
    let end = core::ptr::addr_of_mut!(_bss_end);
    while p < end {
        p.write_volatile(0);
        p = p.add(1);
    }
    crate::rust_main()
}
