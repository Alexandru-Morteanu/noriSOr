#![allow(dead_code)]
//! Registrele procesorului insusi — nu periferice, deci nu stau in hw/.
//! Nu se citesc cu o adresa, ci cu instructiuni dedicate: rsr = read special register.
use core::arch::asm;

/// Unde sare procesorul cand se intampla ceva neasteptat.
/// Acum arata spre ROM. Tot capitolul asta e despre a-l muta la noi.
#[inline(always)]
pub fn vecbase() -> u32 {
    let v: u32;
    unsafe { asm!("rsr.vecbase {0}", out(reg) v, options(nomem, nostack)) };
    v
}

/// Starea procesorului: ferestre active, nivel de intrerupere, mod.
/// Noi am scris 0x00040020 in el la pornire; s-a schimbat de atunci.
#[inline(always)]
pub fn ps() -> u32 {
    let v: u32;
    unsafe { asm!("rsr.ps {0}", out(reg) v, options(nomem, nostack)) };
    v
}

/// Varful stivei. a1 ESTE stiva, prin conventie, pe Xtensa.
#[inline(always)]
pub fn sp() -> u32 {
    let v: u32;
    unsafe { asm!("mov {0}, a1", out(reg) v, options(nomem, nostack)) };
    v
}

/// Muta tabela de vectori la adresa data. Din clipa asta, orice exceptie
/// si orice depasire de fereastra ajung in codul NOSTRU, nu in ROM.
#[inline(always)]
pub unsafe fn set_vecbase(addr: u32) {
    core::arch::asm!(
        "wsr.vecbase {0}",
        "rsync",
        in(reg) addr,
        options(nomem, nostack)
    );
}

macro_rules! citeste {
    ($nume:ident, $reg:literal) => {
        #[inline(always)]
        pub fn $nume() -> u32 {
            let v: u32;
            unsafe { core::arch::asm!(concat!("rsr.", $reg, " {0}"), out(reg) v,
                                      options(nomem, nostack)) };
            v
        }
    };
}

citeste!(exccause, "exccause");
citeste!(epc1,     "epc1");
citeste!(excvaddr, "excvaddr");
citeste!(excsave1, "excsave1");

pub fn nume_cauza(c: u32) -> &'static str {
    match c {
        0  => "instructiune ilegala",
        1  => "apel de sistem",
        2  => "eroare la citirea instructiunii",
        3  => "eroare la acces memorie",
        4  => "intrerupere de nivel 1",
        5  => "alloca",
        6  => "impartire la zero",
        8  => "instructiune privilegiata",
        9  => "acces nealiniat",
        20 => "citire instructiune interzisa",
        28 => "CITIRE de la adresa interzisa",
        29 => "SCRIERE la adresa interzisa",
        _  => "necunoscuta",
    }
}

citeste!(ccount,    "ccount");     // numaratorul de tacturi al procesorului
citeste!(intenable, "intenable");  // ce intreruperi sunt permise
citeste!(interrupt, "interrupt");  // ce intreruperi cer atentie acum

/// Pragul cronometrului intern. Cand CCOUNT il atinge, se declanseaza
/// intreruperea interna numarul 6.
#[inline(always)]
pub unsafe fn set_ccompare0(v: u32) {
    core::arch::asm!("wsr.ccompare0 {0}", "rsync", in(reg) v, options(nomem, nostack));
}

/// Masca intreruperilor permise. Bit 6 = cronometrul intern 0.
#[inline(always)]
pub unsafe fn set_intenable(v: u32) {
    core::arch::asm!("wsr.intenable {0}", "rsync", in(reg) v, options(nomem, nostack));
}
