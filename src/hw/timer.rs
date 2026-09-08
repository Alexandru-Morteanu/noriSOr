use super::{rd, wr, TIMG0};

// Cele 9 registre ale cronometrului 0, la rand de la baza grupului.
const T0CONFIG: usize = TIMG0 + 0x00;
const T0LO:     usize = TIMG0 + 0x04;   // partea de jos a numaratorului (poza)
const T0HI:     usize = TIMG0 + 0x08;   // partea de sus a numaratorului (poza)
const T0UPDATE: usize = TIMG0 + 0x0C;   // scrii aici -> se face poza
const T0LOADLO: usize = TIMG0 + 0x18;   // valoarea de incarcat, partea de jos
const T0LOADHI: usize = TIMG0 + 0x1C;   // valoarea de incarcat, partea de sus
const T0LOAD:   usize = TIMG0 + 0x20;   // scrii aici -> se incarca LOADLO/LOADHI
const REGCLK:   usize = TIMG0 + 0xFC;   // poarta de ceas pentru registre

const EN:        u32 = 1 << 31;  // porneste numaratorul
const INCREASE:  u32 = 1 << 30;  // creste (0 ar insemna sa scada)
const USE_XTAL:  u32 = 1 <<  9;  // sursa = cristalul, NU ceasul procesorului
const DIV_SHIFT: u32 = 13;       // impartitorul incepe de la bitul 13

const XTAL_HZ: u32 = 40_000_000;
const DIVIDER: u32 = XTAL_HZ / 1_000_000;   // 40 -> o bataie = o microsecunda

pub unsafe fn init() {
    // Tinem ceasul registrelor mereu pornit, altfel citirile pot cadea pe langa.
    wr(REGCLK, rd(REGCLK) | (1 << 31));

    // Il oprim inainte sa-l atingem, ca sa nu configuram ceva in miscare.
    wr(T0CONFIG, 0);

    // Il aducem la zero: pui zero in registrele de incarcare,
    // apoi scrii orice in T0LOAD si hardware-ul le muta in numarator.
    wr(T0LOADLO, 0);
    wr(T0LOADHI, 0);
    wr(T0LOAD, 1);

    // Si acum ii dam drumul, cu toate optiunile deodata.
    wr(T0CONFIG, EN | INCREASE | USE_XTAL | (DIVIDER << DIV_SHIFT));
}

/// Cate microsecunde au trecut de cand am pornit cronometrul.
pub unsafe fn now_us() -> u64 {
    // Cerem poza. Scriem si bitul 0 si bitul 31 ca sa nimerim cererea
    // indiferent unde sta ea pe acest cip.
    wr(T0UPDATE, 1 | (1 << 31));
    // Si ASTEPTAM. Hardware-ul sterge singur cererea cand poza e gata.
    // Fara randul asta citesti poza precedenta si masori aiurea.
    while rd(T0UPDATE) != 0 {}
    let lo = rd(T0LO) as u64;      // citeste poza, nu numaratorul viu
    let hi = rd(T0HI) as u64;
    (hi << 32) | lo                // lipim cele doua jumatati
}

/// Asteapta un numar de microsecunde. Prima asteptare adevarata pe care o ai.
pub unsafe fn delay_us(us: u64) {
    let start = now_us();
    while now_us().wrapping_sub(start) < us {}
}
