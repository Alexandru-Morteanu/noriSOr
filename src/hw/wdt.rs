use super::{rd, wr, RTC_CNTL};

const WDTCONFIG0:   usize = RTC_CNTL + 0x98;
const WDTWPROTECT:  usize = RTC_CNTL + 0xB0;
const SWD_CONF:     usize = RTC_CNTL + 0xB4;
const SWD_WPROTECT: usize = RTC_CNTL + 0xB8;

const WDT_KEY: u32 = 0x50D8_3AA1;
const SWD_KEY: u32 = 0x8F1D_312A;
const SWD_AUTO_FEED_EN: u32 = 1 << 31;

/// Oprește ambii câini de pază porniți de ROM la boot.
pub unsafe fn disable() {
    // Câinele obișnuit: descui, oprești, încui la loc.
    wr(WDTWPROTECT, WDT_KEY);
    wr(WDTCONFIG0, 0);
    wr(WDTWPROTECT, 0);

    // Super-câinele nu se poate opri. Îl pui să se hrănească singur.
    wr(SWD_WPROTECT, SWD_KEY);
    let conf = rd(SWD_CONF);
    wr(SWD_CONF, conf | SWD_AUTO_FEED_EN);
    wr(SWD_WPROTECT, 0);
}
