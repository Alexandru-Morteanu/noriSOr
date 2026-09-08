use super::{rd, wr, SYSTEM};

const CPU_PER_CONF: usize = SYSTEM + 0x10;
const SYSCLK_CONF:  usize = SYSTEM + 0x60;

// CPU_PER_CONF, bitii 0:1 -> ce viteza cerem din PLL
const CPUPERIOD_MASK: u32 = 0b11;
const CPUPERIOD_240:  u32 = 2;          // 0=80MHz, 1=160MHz, 2=240MHz
// CPU_PER_CONF, bitul 2 -> pe ce merge PLL-ul
const PLL_480:        u32 = 1 << 2;     // 0=320MHz, 1=480MHz

// SYSCLK_CONF, bitii 10:11 -> de unde isi ia procesorul ceasul
const SOC_CLK_MASK: u32 = 0b11 << 10;
const SOC_CLK_PLL:  u32 = 1 << 10;      // 0=cristal, 1=PLL, 2=oscilator intern

pub unsafe fn cpu_240mhz() {
    // PASUL 1: pregatim destinatia, cat timp inca mergem pe cristal.
    // Citim-modificam-scriem, ca sa nu stergem bitii vecini (waiti etc).
    let mut v = rd(CPU_PER_CONF);
    v |= PLL_480;                                   // 240 se scot doar din 480
    v = (v & !CPUPERIOD_MASK) | CPUPERIOD_240;      // si cerem 240
    wr(CPU_PER_CONF, v);

    // PASUL 2: abia acum intoarcem macazul. De aici incolo fiecare
    // instructiune se executa de sase ori mai repede.
    let mut s = rd(SYSCLK_CONF);
    s = (s & !SOC_CLK_MASK) | SOC_CLK_PLL;
    wr(SYSCLK_CONF, s);
}
