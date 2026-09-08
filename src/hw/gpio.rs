use super::{rd, wr, GPIO, IO_MUX};

// ESP32-S3 imparte pinii in DOUA bancuri. Pinii 0-31 au un set de registre,
// pinii 32-48 au alt set, cu sufix 1. Un "1 << 48" pe u32 nu inseamna nimic
// si intra in panica — de aia fiecare functie alege intai bancul.
const OUT_W1TS:     usize = GPIO + 0x08;
const OUT_W1TC:     usize = GPIO + 0x0C;
const OUT1_W1TS:    usize = GPIO + 0x14;
const OUT1_W1TC:    usize = GPIO + 0x18;
const ENABLE_W1TS:  usize = GPIO + 0x24;
const ENABLE_W1TC:  usize = GPIO + 0x28;
const ENABLE1_W1TS: usize = GPIO + 0x30;
const ENABLE1_W1TC: usize = GPIO + 0x34;
const IN:           usize = GPIO + 0x3C;
const IN1:          usize = GPIO + 0x40;

/// Cuvantul de configurare al unui pad. Sunt asezate la rand, cate 4 octeti.
const fn io_mux(pin: u32) -> usize { IO_MUX + 0x04 + 4 * pin as usize }

const FUN_WPD: u32 = 1 << 7;   // trage firul la masa
const FUN_WPU: u32 = 1 << 8;   // trage firul la 3.3V
const FUN_IE:  u32 = 1 << 9;   // alimenteaza tamponul de CITIRE
const SEL_SHIFT: u32 = 12;     // bitii 12-14: ce periferic primeste firul
const SEL_MASK:  u32 = 0b111 << 12;
const FUNC_GPIO: u32 = 1;      // 1 = controlerul GPIO obisnuit

#[inline(always)] fn banc_sus(pin: u32) -> bool { pin >= 32 }
#[inline(always)] fn masca(pin: u32) -> u32 { 1 << (pin % 32) }

/// Trage firul la controlerul GPIO si porneste citirea.
/// Fara asta, pinul poate fi legat la UART sau SPI si nu raspunde deloc.
pub unsafe fn config(pin: u32) {
    let mut v = rd(io_mux(pin));
    v = (v & !SEL_MASK) | (FUNC_GPIO << SEL_SHIFT);
    v |= FUN_IE;                 // si la iesire: ca sa putem citi inapoi ce am scris
    v &= !(FUN_WPU | FUN_WPD);
    wr(io_mux(pin), v);
}

pub unsafe fn set_output(pin: u32) {
    wr(if banc_sus(pin) { ENABLE1_W1TS } else { ENABLE_W1TS }, masca(pin));
}

pub unsafe fn set_input(pin: u32) {
    wr(if banc_sus(pin) { ENABLE1_W1TC } else { ENABLE_W1TC }, masca(pin));
}

pub unsafe fn high(pin: u32) {
    wr(if banc_sus(pin) { OUT1_W1TS } else { OUT_W1TS }, masca(pin));
}

pub unsafe fn low(pin: u32) {
    wr(if banc_sus(pin) { OUT1_W1TC } else { OUT_W1TC }, masca(pin));
}

/// Ce e pe fir ACUM. La un pin de iesire, citesti inapoi ce ai scris;
/// la unul de intrare, citesti lumea din afara.
pub unsafe fn read(pin: u32) -> bool {
    rd(if banc_sus(pin) { IN1 } else { IN }) & masca(pin) != 0
}

/// Intrare cu rezistenta de tragere in sus. Un buton neapasat nu e "zero",
/// e NIMIC — firul pluteste si citeste zgomot. Rezistenta il tine sus.
/// De aia butonul se citeste invers: 1 = liber, 0 = apasat.
pub unsafe fn intrare_cu_pullup(pin: u32) {
    let mut v = rd(io_mux(pin));
    v = (v & !SEL_MASK) | (FUNC_GPIO << SEL_SHIFT);
    v |= FUN_IE | FUN_WPU;
    v &= !FUN_WPD;               // niciodata amandoua deodata
    wr(io_mux(pin), v);
    set_input(pin);              // si nu conducem noi pinul, ne-am bate cu butonul
}

pub unsafe fn citeste(pin: u32) -> bool { read(pin) }

// Pentru verificare: valorile brute, ca sa nu credem nimic pe cuvant.
pub unsafe fn io_mux_brut(pin: u32) -> u32 { rd(io_mux(pin)) }
pub unsafe fn in_brut() -> u32 { rd(IN) }
