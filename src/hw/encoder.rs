//! Encoder rotativ cu doua contacte decalate (KY-040 / EC11).
//! Directia nu vine dintr-un fir, ci din ORDINEA in care coboara cele doua.
use super::gpio;

#[derive(PartialEq, Clone, Copy)]
pub enum Eveniment { Nimic, Dreapta, Stanga, Apasat, Eliberat }

/// Contactele encoderului sunt rapide: 2 ms de liniste ajung.
const LINISTE_ROTIRE: u32 = 2;
/// Butonul din ax e un contact mare si lenes: ii trebuie 20 ms.
const LINISTE_BUTON:  u32 = 20;

pub struct Encoder {
    clk: u32, dt: u32, sw: u32,
    clk_stabil: bool, clk_citire: bool, clk_moment: u32,
    sw_stabil:  bool, sw_citire:  bool, sw_moment:  u32,
    pub pozitie: i32,
}

impl Encoder {
    pub const fn nou(clk: u32, dt: u32, sw: u32) -> Self {
        Encoder {
            clk, dt, sw,
            clk_stabil: true, clk_citire: true, clk_moment: 0,
            sw_stabil:  true, sw_citire:  true, sw_moment:  0,
            pozitie: 0,
        }
    }

    /// Toate trei sunt intrari trase in sus: contactele leaga la masa
    /// cand se inchid, deci in repaus firul trebuie tinut la 3.3V.
    pub unsafe fn init(&self) {
        gpio::intrare_cu_pullup(self.clk);
        gpio::intrare_cu_pullup(self.dt);
        gpio::intrare_cu_pullup(self.sw);
    }

    /// Se cheama des, cu numarul de milisecunde de la TICKS.
    pub unsafe fn actualizeaza(&mut self, ms: u32) -> Eveniment {
        // --- rotirea ---
        let c = gpio::citeste(self.clk);
        if c != self.clk_citire {
            self.clk_citire = c;
            self.clk_moment = ms;               // a sarit, reincepem numaratoarea
        } else if c != self.clk_stabil && ms.wrapping_sub(self.clk_moment) >= LINISTE_ROTIRE {
            self.clk_stabil = c;
            if !c {
                // Front cazator curat pe CLK. ACUM ne uitam la DT:
                // el nu a apucat inca sa se schimbe, deci ne spune directia.
                return if gpio::citeste(self.dt) {
                    self.pozitie += 1;
                    Eveniment::Dreapta
                } else {
                    self.pozitie -= 1;
                    Eveniment::Stanga
                };
            }
        }

        // --- butonul din ax ---
        let b = gpio::citeste(self.sw);
        if b != self.sw_citire {
            self.sw_citire = b;
            self.sw_moment = ms;
        } else if b != self.sw_stabil && ms.wrapping_sub(self.sw_moment) >= LINISTE_BUTON {
            self.sw_stabil = b;
            return if b { Eveniment::Eliberat } else { Eveniment::Apasat };
        }

        Eveniment::Nimic
    }
}
