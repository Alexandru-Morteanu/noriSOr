#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod boot;
mod cpu;
mod hw;

use core::panic::PanicInfo;

extern "C" {
    static _vecbase: u8;   // pus de linker la inceputul sectiunii .vectors
}

/// Bataia de inima. Creste cu 1 la fiecare milisecunda, din assembler,
/// fara ca bucla principala sa stie ceva. Sta in .bss, deci porneste de la 0.
#[no_mangle]
pub static mut TICKS: u32 = 0;

unsafe fn linie(nume: &str, val: u32, ce: &str) {
    hw::usb_serial::puts(nume);
    hw::usb_serial::put_hex(val);
    hw::usb_serial::puts("   <- ");
    hw::usb_serial::puts(ce);
    hw::usb_serial::puts("\r\n");
}

/// Proba de foc pentru ferestrele de registre: la 40 de nivele fereastra
/// TREBUIE sa se verse pe stiva si sa se intoarca. Daca e bine, iese 820.
#[inline(never)]
unsafe fn adanc(n: u32) -> u32 {
    if n == 0 { return 0; }
    let r = adanc(n - 1);
    r.wrapping_add(n)
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    unsafe {
        hw::timer::init();
        hw::clock::cpu_240mhz();
        hw::timer::delay_us(3_000_000);

        hw::usb_serial::puts("\r\n=== INAINTE DE COMUTARE ===\r\n");
        linie("VECBASE = ", cpu::vecbase(), "tabela din ROM");

        let tabela = core::ptr::addr_of!(_vecbase) as u32;
        linie("tabela noastra = ", tabela, "acolo mutam");

        cpu::set_vecbase(tabela);

        hw::usb_serial::puts("\r\n=== DUPA COMUTARE ===\r\n");
        linie("VECBASE = ", cpu::vecbase(), "acum e a noastra");

        hw::usb_serial::puts("proba de adancime (40 nivele) ... ");
        let r = adanc(40);
        hw::usb_serial::put_u32(r);
        if r == 820 {
            hw::usb_serial::puts("   CORECT\r\n\r\n");
        } else {
            hw::usb_serial::puts("   GRESIT, ar fi trebuit 820\r\n\r\n");
        }

        hw::usb_serial::puts("pornesc cronometrul intern al procesorului.\r\n");
        hw::usb_serial::puts("peste ~0.5 secunde ma va intrerupe in mijlocul buclei.\r\n\r\n");

        // CCOUNT numara tacturi la viteza procesorului: 240 de milioane pe
        // secunda. Punem pragul la jumatate de secunda in viitor.
        let acum = cpu::ccount();
        cpu::set_ccompare0(acum.wrapping_add(120_000_000));

        // Si deschidem poarta: bit 6 = cronometrul intern 0.
        // Din clipa asta procesorul poate fi intrerupt oricand.
        cpu::set_intenable(1 << 6);

        hw::usb_serial::puts("configurez GPIO 48 (beculetul de pe placa) ca iesire.\r\n");
        hw::gpio::config(48);
        hw::gpio::set_output(48);

        hw::gpio::high(48);
        linie("GPIO48 dupa high = ", hw::gpio::read(48) as u32, "1 = citim exact ce am scris");
        hw::gpio::low(48);
        linie("GPIO48 dupa low  = ", hw::gpio::read(48) as u32, "0 = citim exact ce am scris");

        hw::usb_serial::puts("\r\nATENTIE: pinul 48 de pe DevKitC-1 e un LED WS2812\r\n");
        hw::usb_serial::puts("(adresabil), nu un bec simplu -- un HIGH/LOW nu-l aprinde\r\n");
        hw::usb_serial::puts("vizibil, ii trebuie un protocol cu impulsuri de ~1us.\r\n");
        hw::usb_serial::puts("Proba de mai sus verifica doar ca driverul scrie/citeste\r\n");
        hw::usb_serial::puts("corect -- vezi capcana din README.\r\n\r\n");

        // --- B1: encoder rotativ pe GPIO 4 (CLK), 5 (DT), 6 (SW) ---
        let mut enc = hw::encoder::Encoder::nou(4, 5, 6);
        enc.init();

        hw::usb_serial::puts("roteste butonul. apasa-l ca sa vezi ce iese.\r\n\r\n");
        let mut volum: i32 = 10;
        bara(volum);

        loop {
            let ms = core::ptr::addr_of!(TICKS).read_volatile();
            match enc.actualizeaza(ms) {
                hw::encoder::Eveniment::Dreapta => {
                    if volum < 20 { volum += 1; }
                    bara(volum);
                }
                hw::encoder::Eveniment::Stanga => {
                    if volum > 0 { volum -= 1; }
                    bara(volum);
                }
                hw::encoder::Eveniment::Apasat => {
                    hw::usb_serial::puts("   [apasat]  pozitie bruta = ");
                    hw::usb_serial::put_u32(enc.pozitie as u32);
                    hw::usb_serial::puts("\r\n");
                }
                _ => {}
            }
            hw::timer::delay_us(500);
        }
    }
}

unsafe fn bara(v: i32) {
    hw::usb_serial::puts("volum ");
    if v < 10 { hw::usb_serial::puts(" "); }
    hw::usb_serial::put_u32(v as u32);
    hw::usb_serial::puts("  [");
    let mut i = 0;
    while i < 20 {
        hw::usb_serial::puts(if i < v { "#" } else { "." });
        i += 1;
    }
    hw::usb_serial::puts("]\r\n");
}

/// Aici ajunge orice exceptie SI orice intrerupere de nivel 1 — pe Xtensa
/// intra pe aceeasi usa, se deosebesc prin cauza. Vectorul a curatat deja
/// PS, deci aici avem voie sa apelam functii normale.
#[no_mangle]
pub extern "C" fn _exc_report() -> ! {
    unsafe {
        let c = cpu::exccause();
        hw::usb_serial::puts("\r\n########  OPRIRE  ########\r\n");
        hw::usb_serial::puts("cauza    = ");
        hw::usb_serial::put_u32(c);
        hw::usb_serial::puts("  ");
        hw::usb_serial::puts(cpu::nume_cauza(c));
        hw::usb_serial::puts("\r\n");

        linie("unde     = ", cpu::epc1(), "unde a fost oprit procesorul");

        // EXCVADDR e completat DOAR la erori de memorie. La orice altceva
        // contine ce a ramas de dinainte — mai bine tacem decat sa mintim.
        if matches!(c, 2 | 3 | 9 | 20 | 24 | 25 | 26 | 28 | 29) {
            linie("ce adresa= ", cpu::excvaddr(), "adresa de memorie ceruta");
        }

        // a0 tine adresa de intoarcere DOAR in bitii 0..29. Bitii 30-31 sunt
        // tipul apelului (call4 / call8 / call12), acelasi CALLINC din PS.
        let a0 = cpu::excsave1();
        linie("tip apel = ", a0 >> 30, "1=call4  2=call8  3=call12");
        linie("intoarcere=", (a0 & 0x3FFF_FFFF) | 0x4000_0000, "cine a apelat, adresa curatata");

        hw::usb_serial::puts("##########################\r\n");
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
