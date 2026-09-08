# player — un sistem de operare scris de la zero

Sistem de operare bare-metal pentru **ESP32-S3**, scris în **Rust + assembler Xtensa**,
fără ESP-IDF, fără FreeRTOS, fără `esp-hal`, fără nicio dependență.

Scopul final e un player MP3 fizic. Scopul real e să înțeleg cum funcționează un
sistem de operare construindu-l.

| | |
|---|---|
| Dependențe | **0** |
| Rânduri sursă | **367** |
| Încărcat în RAM | **2 596 octeți** |
| Ceas CPU | **240 MHz** (verificat, de la 20 MHz) |

---

## Regulile jocului

**Nu folosim:** ESP-IDF · FreeRTOS · `esp-hal` sau orice crate de hardware ·
bootloader-ul Espressif · biblioteca standard `std`.

**Folosim:** `core` · assembler scris de mână · script de linker propriu ·
manualul cipului · OpenOCD, doar ca să încărcăm codul.

Un crate de hardware ar rezolva fiecare pas de mai jos într-o linie. Ar merge mai
repede și n-aș ști nimic.

---

## Placa

ESP32-S3 DevKitC-1, varianta N16R8.

| Ce | Valoare | De ce contează |
|---|---|---|
| Nuclee | 2 × Xtensa LX7 | Al doilea stă în reset — se pornește la etapa D |
| Cristal | 40 MHz | Sursa de adevăr pentru orice măsurătoare de timp |
| RAM intern | 512 KB | Tot programul stă aici; flash-ul nu e atins |
| Pini interziși | GPIO 26–32 | Legați la flash-ul intern — atinși, cipul moare |
| Pini rezervați | GPIO 33–37 | PSRAM octal pe varianta R8 |
| USB nativ | GPIO 19 / 20 | Un singur cablu: și programare, și consolă |

---

## Cum rulezi

```bash
./run.sh
```

Atât. Scriptul: eliberează portul USB → `cargo build` → citește punctul de intrare
din ELF → încarcă în RAM prin JTAG și dă drumul → deschide consola.
Ieși cu `Ctrl-C`.

Nu se scrie nimic în flash. Codul trăiește doar în RAM: se pierde la deconectare,
dar se încarcă în 20 ms și nu are nimic al Espressif în față.

> **Punctul de intrare se citește din ELF, nu se scrie de mână.** Se mută de fiecare
> dată când crește codul, pentru că tabelul de constante dinaintea lui `_start`
> crește odată cu el. O adresă scrisă de mână a costat o oră de depanare.

---

## Structura

```
player/
├── linker.ld            48 r.  unde ajunge fiecare bucată în memorie
├── build.rs              6 r.  recompilează când se schimbă linker.ld
├── run.sh               38 r.  compilează → încarcă → rulează → ascultă
└── src/
    ├── main.rs          49 r.  demonstrația: măsoară, comută ceasul, măsoară
    ├── boot/
    │   └── start.rs     47 r.  assembler de la reset + ștergerea .bss
    └── hw/
        ├── mod.rs       18 r.  adrese de bază + rd() / wr()
        ├── wdt.rs       24 r.  oprește cei doi câini de pază
        ├── usb_serial.rs 53 r. text pe USB, octet cu octet
        ├── timer.rs     55 r.  cronometru 1 MHz legat la cristal
        ├── clock.rs     29 r.  ceasul CPU: cristal → PLL
        └── gpio.rs       0 r.  gol — etapa B
```

### Modelul mental: un registru este o adresă

Tot proiectul stă pe două funcții:

```rust
pub unsafe fn wr(addr: usize, val: u32) { (addr as *mut u32).write_volatile(val); }
pub unsafe fn rd(addr: usize) -> u32    { (addr as *const u32).read_volatile() }
```

`volatile` e cuvântul important — interzice compilatorului să „optimizeze”
citirile și scrierile. Fără el, jumătate din cod ar dispărea la compilare.

---

## Pornirea

```
RESET ──▶ ROM din fabrică ──▶ _start ──▶ _start_rust ──▶ rust_main
          0x4000_0400        (asm)      (primul Rust)   (programul)
```

| Verigă | Ce lasă în urmă |
|---|---|
| ROM | pornește câinii de pază · împarte cristalul la 2 (CPU 20 MHz) · trezește PLL-ul |
| `_start` | fereastra de registre în stare cunoscută · `PS.WOE = 1` · stiva în `a1` |
| `_start_rust` | omoară câinii de pază · șterge `.bss` cu zerouri |

### De ce cele șase instrucțiuni sunt obligatorii

Xtensa are *ferestre de registre*: fiecare funcție primește una prin instrucțiunea
`entry`, pe care Rust o pune la începutul fiecărei funcții compilate. Dar `entry`
este **ilegală** dacă bitul WOE din registrul de stare e zero — cum e imediat după
reset. Fără inițializarea de mai jos, prima funcție Rust apelată aruncă o excepție
și cipul se întoarce mut în ROM.

```asm
_start:
  movi a0, 0
  wsr.windowbase a0     ; fereastra 0
  movi a0, 1
  wsr.windowstart a0    ; doar ea e validă
  rsync
  movi a0, 0x00040020   ; bit 18 = WOE, bit 5 = mod utilizator
  wsr.ps a0
  rsync
  movi a0, 0            ; fără adresă de întoarcere
  movi a1, _stack_top   ; stiva
  j    _start_rust
```

---

## Memoria — aceeași memorie, două uși

Același RAM fizic apare de două ori în harta de adrese. Prin ușa de execuție
procesorul poate rula cod dar nu poate citi date byte cu byte; prin ușa de date
poate citi și scrie dar nu poate executa.

```
  UȘA DE EXECUȚIE (IRAM)              UȘA DE DATE (DRAM)
  0x4037_8000 ┌──────────────┐        0x3FCA_8000 ┌──────────────┐
              │ .text        │                    │ .rodata      │
              │ 2 020 octeți │                    │ .data        │
              ├──────────────┤                    ├──────────────┤
              │              │   același          │ .bss         │
              │ liber        │◀── siliciu ──▶     │ 576 octeți   │
              │ ≈ 126 KB     │                    ├──────────────┤
              │              │                    │ liber        │
              │              │                    ├──────────────┤
              │              │                    │ STIVA  ↓     │
  0x4039_8000 └──────────────┘        0x3FCC_8000 └──────────────┘
                                                   _stack_top
```

Nimeni nu verifică dacă stiva coborâtă lovește datele. Când se va întâmpla, cipul
va tăcea — de asta urmează tabela de vectori.

---

## Ceasurile — rigla trebuie să stea pe loc

```
                    ┌── ÷2 ──────────────────▶ CPU 20 MHz     (înainte)
                    │
  CRISTAL 40 MHz ───┼── PLL 480 MHz ── ÷2 ───▶ CPU 240 MHz    (acum)
    (cuarț fizic)   │
                    └── ÷40 ─────────────────▶ TIMER 1 MHz    (rigla)
                                                1 bătaie = 1 µs
```

Cronometrul putea fi legat la ceasul procesorului. Ar fi fost mai simplu și
complet inutil: când urci ceasul se schimbă și rigla odată cu obiectul măsurat.
E legat la cristal — **bitul 9 din `T0CONFIG`**.

### `T0CONFIG` = `0xC005_0200`

| Biți | Câmp | Valoare |
|---|---|---|
| 31 | pornit | 1 |
| 30 | numără în sus | 1 |
| 28:13 | împărțitor | 40 → 40 MHz ÷ 40 = **1 MHz** |
| 9 | sursa = cristal | 1 |

### Ordinea la comutarea ceasului contează

Cei 240 MHz se obțin doar împărțind 480 la doi — din PLL-ul de 320 MHz nu iese 240
cu niciun împărțitor întreg. Deci: **întâi pregătești destinația, apoi întorci
macazul.** Invers, procesorul cere o viteză imposibilă și se oprește la jumătatea
unei instrucțiuni.

> Trezirea PLL-ului de la zero ar fi cerut scrieri pe o magistrală I²C internă cu
> numere magice. N-a fost nevoie: portul USB pe care scoteam text are nevoie de
> 48 MHz, iar singura sursă e PLL-ul. Faptul că textul ieșea deja era dovada că
> PLL-ul e pornit.

---

## Dovada

| Moment | 100 000 `nop` | Ceas CPU | Tacturi totale | Tacturi / rotire |
|---|---|---|---|---|
| Pe cristal | 25 003 µs | 20 MHz | 500 060 | **5,00** |
| Pe PLL | 2 084 µs | 240 MHz | 500 160 | **5,00** |

Raportul e exact 12,0. Iar când împarți fiecare durată la ceasul presupus, ambele
dau **același număr de tacturi pe rotire de buclă: 5,0** — un `nop`, incrementarea
contorului, comparația, saltul înapoi. Două măsurători independente care cad pe
aceeași constantă întreagă nu mai sunt noroc.

Asta confirmă simultan că procesorul merge la 240 MHz *și* că înainte mergea la 20
— lucru pe care nu-l știam: ROM-ul împărțea cristalul la doi fără să spună nimănui.

---

## Capcane

| Simptom | Ce era de fapt | Reparație |
|---|---|---|
| `dangerous relocation: l32r` | `l32r` poate privi doar înapoi; `.literal` nu era adunat în linker | `*(.literal .text .literal.* .text.*)` |
| `Finished in 0.24s` după ce editez `linker.ld` | cargo nu știe de linker | `build.rs` cu `rerun-if-changed` |
| `espflash: App Descriptor missing` | unealta cere antetul Espressif | ocolit — încărcare în RAM prin OpenOCD |
| Se termină mereu la `PC=0x400003C0` | `entry` ilegală cu `PS.WOE = 0` | cele 6 instrucțiuni din `_start` |
| `reg pc 0x40378008` — nu era codul acolo | tabelul de constante a crescut, `_start` s-a mutat | adresa se citește din ELF |
| `mdw` arată o valoare „care merge” | gunoi din RAM neinițializat | ștergerea `.bss`; citește orice valoare de două ori |
| `esp_usb_jtag: could not find device` | un port, două programe | OpenOCD ca o comandă unică terminată cu `shutdown` |
| Text apare, apoi se oprește | `halt` în script îngheța CPU-ul înainte de consolă | scos `halt` |
| `100000 nop = 3 us` | poza cronometrului citită înainte să fie gata | `while rd(T0UPDATE) != 0 {}` |

**Nerezolvat:** GDB nu se conectează la OpenOCD (`Remote 'g' packet reply is too
long`) — nepotrivire de versiuni. Ocolit prin comenzi directe către OpenOCD.

---

## Drumul

- [x] **A1** — unelte: toolchain Rust pentru Xtensa, `xtensa-esp32s3-none-elf`, OpenOCD
- [x] **A2** — pornire proprie: linker, assembler de la reset, ștergerea `.bss`
- [x] **A3.1** — câinii de pază opriți
- [x] **A3.2** — primul text pe USB, driver scris de la zero
- [x] **A3.3** — cronometru 1 MHz pe cristal + ceas CPU la 240 MHz
- [ ] **A3.4** — **tabela de vectori**: întreruperi, excepții, bătaie de inimă la 1000 Hz
- [ ] **A4** — antet de imagine propriu, ca placa să pornească singură din flash
- [ ] **B** — butoane, LED-uri, GPIO
- [ ] **C** — alocator de memorie, card SD, FAT32
- [ ] **D** — planificator de sarcini, al doilea nucleu
- [ ] **E** — decodor MP3, I²S, sunet pe căști

### De ce vectorii, și nu butoanele

Acum programul întreabă. Când va trebui hrănit un decodor de sunet de 44 100 de ori
pe secundă, în timp ce cardul SD e citit și butoanele sunt urmărite, întrebatul pe
rând nu mai ajunge — se pierd bătăi și se aud pocnituri.

Tabela de vectori întoarce relația: hardware-ul întrerupe programul. La nivelul
siliciului o „întrerupere” înseamnă un singur lucru — procesorul sare la o adresă
fixă. Tabela e lista acelor adrese; acum e goală și arată spre ROM. De asta, când
programul greșește, cipul îngheață mut în loc să spună unde.

---

## Comenzi utile

```bash
ELF=~/.cargo-target/player/xtensa-esp32s3-none-elf/debug/player

./run.sh                                        # tot ciclul
xtensa-esp32s3-elf-readelf -S $ELF              # unde a ajuns fiecare secțiune
xtensa-esp32s3-elf-nm      $ELF | grep NUME     # adresa unei variabile
xtensa-esp32s3-elf-objdump -d $ELF              # codul mașină generat
lsof | grep usbmodem                            # cine ține portul ocupat
```

Adresele și biții registrelor se scot din sursa descrisă a registrelor din cache-ul
cargo — folosită ca înlocuitor de manual, nu ca dependență: din ea copiem numere,
nu cod.

```bash
PAC=$(echo ~/.cargo/registry/src/index.crates.io-*/esp32s3-*/src)
grep -n -B4 'fn NUME(' $PAC/system.rs      # offsetul registrului
grep -n 'doc = "Bit'   $PAC/system/NUME.rs # câmpurile de biți
```
