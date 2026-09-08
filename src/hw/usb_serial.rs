use super::{rd, wr, USB_DEV};

const EP1:      usize = USB_DEV + 0x00;  // cutia poștală
const EP1_CONF: usize = USB_DEV + 0x04;  // starea ei

const DATA_FREE: u32 = 1 << 1;  // citit: mai e loc
const WR_DONE:   u32 = 1 << 0;  // scris: gata, trimite

/// Pune un octet în cutie. Dacă nimeni nu ascultă, renunță după un timp.
pub unsafe fn putb(b: u8) {
    let mut spin: u32 = 0;
    while rd(EP1_CONF) & DATA_FREE == 0 {
        spin += 1;
        if spin > 200_000 {
            return;
        }
    }
    wr(EP1, b as u32);
}

/// Trimite ce s-a adunat în cutie.
pub unsafe fn flush() {
    wr(EP1_CONF, WR_DONE);
}

/// Scrie un text.
pub unsafe fn puts(s: &str) {
    for b in s.bytes() {
        putb(b);
    }
    flush();
}

/// Scrie un număr în zecimal. Fără biblioteci — de la zero.
pub unsafe fn put_u32(mut v: u32) {
    if v == 0 {
        putb(b'0');
        flush();
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while v > 0 {
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        putb(buf[i]);
    }
    flush();
}
