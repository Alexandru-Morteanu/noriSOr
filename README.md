# player — un sistem de operare scris de la zero

Sistem de operare bare-metal pentru **ESP32-S3**, scris în **Rust + assembler Xtensa**,
fără ESP-IDF, fără FreeRTOS, fără `esp-hal`, fără nicio dependență.

Scopul final e un player MP3 fizic. Scopul real e să înțeleg cum funcționează un
sistem de operare construindu-l.

| | |
|---|---|
| Dependențe | **0** |
| Rânduri sursă | **900** |
| Încărcat în RAM | **10 952 octeți** (înainte de GPIO — de reverificat) |
| Ceas CPU | **240 MHz** (de la 20, verificat de trei ori) |
| Bătaie de sistem | **1000 Hz**, fără derivă |

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
dar se încarcă în 90 ms și nu are nimic al Espressif în față.

> **Punctul de intrare se citește din ELF, nu se scrie de mână.** Se mută de fiecare
> dată când crește codul, pentru că tabelul de constante dinaintea lui `_start`
> crește odată cu el. O adresă scrisă de mână a costat o oră de depanare.

---

## Structura

```
player/
├── linker.ld            62 r.  unde ajunge fiecare bucată în memorie
├── build.rs             25 r.  asamblează vectorii, recompilează la nevoie
├── run.sh               38 r.  compilează → încarcă → rulează → ascultă
└── src/
    ├── main.rs         140 r.  demonstrația + raportul de excepții
    ├── cpu.rs           94 r.  registrele procesorului însuși (rsr / wsr)
    ├── boot/
    │   ├── start.rs     47 r.  assembler de la reset + ștergerea .bss
    │   └── vectors.S   191 r.  tabela de vectori + handlerele de fereastră
    └── hw/
        ├── mod.rs       19 r.  adrese de bază + rd() / wr()
        ├── wdt.rs       24 r.  oprește cei doi câini de pază
        ├── usb_serial.rs 53 r. text pe USB, octet cu octet
        ├── timer.rs     55 r.  cronometru 1 MHz legat la cristal
        ├── clock.rs     29 r.  ceasul CPU: cristal → PLL
        └── gpio.rs     123 r.  IO_MUX (funcția pad-ului) + GPIO (nivel, voie de ieșire)
```

### Modelul mental: un registru este o adresă

Perifericele se ating prin două funcții:

```rust
pub unsafe fn wr(addr: usize, val: u32) { (addr as *mut u32).write_volatile(val); }
pub unsafe fn rd(addr: usize) -> u32    { (addr as *const u32).read_volatile() }
```

`volatile` e cuvântul important — interzice compilatorului să „optimizeze”
citirile și scrierile. Fără el, jumătate din cod ar dispărea la compilare.

Registrele **procesorului însuși** (`PS`, `VECBASE`, `EPC1`, `CCOUNT`…) nu au
adrese. Se ajunge la ele doar cu instrucțiuni dedicate — `rsr` / `wsr` — de aceea
stau separat, în `cpu.rs`.

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
              │ .vectors 1KB │                    │ .rodata      │
  0x4037_8400 ├──────────────┤                    │ .data        │
              │ .text        │                    ├──────────────┤
              │              │   același          │ .bss (TICKS) │
              ├──────────────┤◀── siliciu ──▶     ├──────────────┤
              │ liber        │                    │ liber        │
              │              │                    ├──────────────┤
              │              │                    │ STIVA  ↓     │
  0x4039_8000 └──────────────┘        0x3FCC_8000 └──────────────┘
                                                   _stack_top
```

Nimeni nu verifică dacă stiva coborâtă lovește datele — și nimic nu păzește
memoria în general (vezi capcane). Un pointer greșit nu crapă aici, strică
în tăcere.

---

## Ceasurile — rigla trebuie să stea pe loc

```
                    ┌── ÷2 ──────────────────▶ CPU 20 MHz     (înainte)
                    │
  CRISTAL 40 MHz ───┼── PLL 480 MHz ── ÷2 ───▶ CPU 240 MHz    (acum)
    (cuarț fizic)   │                            │
                    │                            └── CCOUNT ──▶ bătaie 1000 Hz
                    │
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
macazul.**

> Trezirea PLL-ului de la zero ar fi cerut scrieri pe o magistrală I²C internă cu
> numere magice. N-a fost nevoie: portul USB pe care scoteam text are nevoie de
> 48 MHz, iar singura sursă e PLL-ul. Faptul că textul ieșea deja era dovada că
> PLL-ul e pornit.

---

## Vectorii — cine răspunde când se întâmplă ceva

Un procesor nu „raportează erori”. Când ceva iese din tipar, **sare la o adresă
fixă**. Lista acelor adrese e tabela de vectori, iar `VECBASE` spune unde e ea.
La pornire arată în ROM. Acum arată la noi:

```
VECBASE: 0x4000_0000 (ROM)  ───▶  0x4037_8000 (al nostru)
```

### Harta tabelei — 1 KB, poziții impuse de siliciu

| Offset | Vector | Stare |
|---|---|---|
| `+0x000` … `+0x140` | 6 × overflow / underflow de fereastră | **scrise de noi** |
| `+0x180` … `+0x2C0` | întreruperi nivel 2–5, debug, NMI | buclă pe loc |
| `+0x300` | excepție de kernel | buclă pe loc |
| `+0x340` | **excepție de utilizator** | **scrisă de noi** |
| `+0x3C0` | excepție dublă | buclă pe loc |

`VECBASE` nu poate arăta decât la un multiplu de 1024, iar în interior fiecare
vector are locul lui fix. Nu le alegem noi, le nimerim.

### Prețul ferestrelor

`PS.WOE = 1`, pus la pornire, are o consecință: când apelurile se cuibăresc destul
cât să epuizeze cele 64 de registre fizice, procesorul **varsă automat fereastra
cea mai veche pe stivă** printr-o excepție. Cât timp `VECBASE` arăta în ROM, treaba
asta o făcea codul din fabrică. În clipa în care mutăm `VECBASE`, o preluăm noi —
și nu se poate pe jumătate.

Cele șase handlere sunt specificate de arhitectură; există exact un răspuns corect,
aceleași instrucțiuni ca în FreeRTOS sau Zephyr. `s32e` / `l32e` există în siliciu
**doar** pentru asta: „scrie în cadrul de stivă al ferestrei vecine”.

Proba: o funcție recursivă pe 40 de nivele care întoarce `820`. Ca să adune 1…40,
procesorul a vărsat ferestre pe stivă coborând și le-a încărcat urcând, de vreo
cinci ori în fiecare direcție, prin cod propriu.

### Excepție sau întrerupere — aceeași ușă

Pe Xtensa, **întreruperile de nivel 1 intră prin vectorul de excepție de
utilizator**, cu `EXCCAUSE = 4`. Deci o singură ușă, două drumuri:

```
                            ┌─ cauza = 4 ─▶ _int_tick ─▶ TICKS+1 ─▶ rfe ─┐
  ceva se întâmplă ─▶ _vec_user                                          │
                            └─ altceva  ─▶ _exc_report ─▶ raport ─▶ stop │
                                                                         ▼
                                                      înapoi exact unde a fost oprit
```

### Raportul de excepție

```
########  OPRIRE  ########
cauza    = 0  instructiune ilegala
unde     = 0x4037a018   <- unde a fost oprit procesorul
tip apel = 2            <- 1=call4  2=call8  3=call12
intoarcere=0x4037a321   <- cine a apelat, adresa curatata
##########################
```

Adresa de la `unde` se caută direct în `objdump -d` și îți arată linia ta de cod.
Prima oară în tot proiectul când cipul spune unde a greșit, în loc să tacă.

### Bătaia de inimă — 1000 Hz, fără derivă

Handlerul de bătaie e scris **numai în assembler** și nu cheamă nicio funcție Rust:
orice funcție Rust începe cu `entry`, care rotește fereastra. `EXCM` rămâne pe 1 în
tot handlerul, ceea ce blochează atât alte întreruperi, cât și excepțiile de
fereastră. Atinge trei registre, le pune la loc, pleacă prin `rfe`.

```asm
    rsr.ccompare0 a2        ; pragul urmator = pragul VECHI + pas
    l32r    a3, .Lpas       ; nu CCOUNT + pas — altfel intarzierile
    add     a2, a2, a3      ; se aduna si ceasul ramane in urma
    wsr.ccompare0 a2        ; scrierea sterge si cererea de intrerupere
```

---

## Dovada

| Moment | 100 000 `nop` | Ceas CPU | Tacturi totale | Tacturi / rotire |
|---|---|---|---|---|
| Pe cristal | 25 003 µs | 20 MHz | 500 060 | **5,00** |
| Pe PLL | 2 084 µs | 240 MHz | 500 160 | **5,00** |

Raportul e exact 12,0. Iar când împarți fiecare durată la ceasul presupus, ambele
dau **același număr de tacturi pe rotire: 5,0** — un `nop`, incrementarea
contorului, comparația, saltul înapoi. Două măsurători independente care cad pe
aceeași constantă întreagă nu mai sunt noroc.

**A treia confirmare, independentă de primele două:**

```
batai = 1501   (+1000 in ultima secunda)
batai = 2501   (+1000 in ultima secunda)
batai = 3501   (+1000 in ultima secunda)
```

Bătăile sunt numărate de cronometrul **din procesor** (PLL, 240 MHz). Secunda în
care sunt numărate e măsurată de cronometrul **TIMG0** (cristal, 40 MHz). Două
ceasuri care nu comunică între ele, care cad pe același adevăr.

---

## GPIO — două uși pentru un pad

Un pad fizic nu e „automat” GPIO. Trece prin două periferice diferite, la
două adrese diferite, și fiecare are treaba lui:

```
IO_MUX (0x6000_9000)              GPIO (0x6000_4000)
alege FUNCȚIA pad-ului      ──▶   aprinde / stinge / citește nivelul
"funcția 1" = GPIO simplu         doar dacă IO_MUX l-a lăsat liber
```

### IO_MUX — un registru pe pad, la pas fix

Cele 49 de pad-uri au fiecare registrul lui, la `IO_MUX + 0x04 + 4×n` —
confirmat direct din structura crate-ului (`gpio(n)`, plaja documentată
`0x04..0xc8` pentru 49 de intrări de 4 octeți fiecare). Câmpul care
contează acum:

| Biți | Câmp | Ce face |
|---|---|---|
| 12:14 | `MCU_SEL` | funcția pad-ului. 0 = „Funcția 1" — pe S3, uniform GPIO simplu |
| 9 | `FUN_IE` | intrarea spre chip, activată |
| 10:11 | `FUN_DRV` | tăria semnalului (0=~5mA … 3=~40mA) |

### GPIO — jumătate de registru pentru fiecare 32 de pini

Moștenire de pe ESP32 original: perifericul GPIO împarte cei 49 de pini în
două jumătăți, cu registre separate:

| | Pinii 0-31 | Pinii 32-53 |
|---|---|---|
| Nivel | `OUT` (+0x04) | `OUT1` (+0x10) |
| Voie de ieșire | `ENABLE` (+0x20) | `ENABLE1` (+0x2c) |
| Citire | `IN` (+0x3c) | `IN1` (+0x40) |

Fiecare are perechea ei `_W1TS` / `_W1TC` („write 1 to set/clear") — scrii
doar bitul pinului tău, restul rămân neatinși. Fără ele, ai avea nevoie de
citește-modifică-scrie, iar două treburi care ating pini diferiți din
același registru s-ar putea călca în picioare.

Piciorusul **48** (beculețul de pe placă) cade în a doua jumătate — de-aici
`OUT1` / `ENABLE1` / `IN1` în driver, în loc de variantele simple.

### Proba — fără niciun fir extra

Configurez pinul 48 ca ieșire, îl ridic, citesc înapoi ce am scris (`read`
merge și pe o ieșire — arată nivelul fizic al pad-ului, nu „ce am cerut"),
îl cobor, citesc iar:

```
GPIO48 dupa high = 00000001   <- 1 = citim exact ce am scris
GPIO48 dupa low  = 00000000   <- 0 = citim exact ce am scris
```

Nu se vede niciun bec clipind — și nu e o greșeală, e prima capcană de mai jos.

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
| `iram overflowed by 1077252740 bytes` | în interiorul unei secțiuni, `.` e **deplasare**, nu adresă | `. = 0x040;`, nu `. = _vecbase + 0x040;` |
| `s32e: instruction use requires an option` | asamblorul din LLVM nu activează opțiunea de ferestre | vectorii într-un `.S`, asamblat de `xtensa-esp32s3-elf-gcc` din `build.rs` |
| Citirea de la adresa `0` **nu** dă eroare | nimic nu păzește memoria pe acest cip | folosește `ill` ca test; protecția memoriei e un periferic separat, nepornit |
| Șiruri Rust rupte după un patch | `re.sub` din Python **procesează** `\r\n` din textul de înlocuire | `str.replace`, sau rescrie fișierul întreg |
| GPIO48 arată corect în citire, dar nu clipește vizibil | e un LED **WS2812** (adresabil), nu un bec simplu — vrea un protocol serial cu impulsuri de ~1 µs, nu un nivel static | de făcut mai târziu: bit-banging WS2812 pe CCOUNT, sau un LED extern pe un pin liber pentru probe rapide |

**Două lucruri de reținut din raportul de excepție:** `EXCVADDR` e completat *doar*
la erori de memorie — la orice altceva conține gunoi vechi. Și `a0` ține adresa de
întoarcere doar în biții 0–29; biții 30–31 sunt tipul apelului (`call4` / `call8` /
`call12`), aceeași informație din câmpul CALLINC al lui `PS`.

**Cauza 0 la o adresă** înseamnă aproape întotdeauna „am ajuns unde nu trebuia”:
instrucțiunea ilegală pe Xtensa e literalmente `000000`, adică memorie goală.

**Nerezolvat:** GDB nu se conectează la OpenOCD (`Remote 'g' packet reply is too
long`) — nepotrivire de versiuni. Ocolit prin comenzi directe către OpenOCD.

---

## Drumul

- [x] **A1** — unelte: toolchain Rust pentru Xtensa, `xtensa-esp32s3-none-elf`, OpenOCD
- [x] **A2** — pornire proprie: linker, assembler de la reset, ștergerea `.bss`
- [x] **A3.1** — câinii de pază opriți
- [x] **A3.2** — primul text pe USB, driver scris de la zero
- [x] **A3.3** — cronometru 1 MHz pe cristal + ceas CPU la 240 MHz
- [x] **A3.4** — tabela de vectori proprie, handlere de fereastră, raport de excepții, bătaie 1000 Hz
- [x] **B1** — driver GPIO: `IO_MUX` (funcția pad-ului) + `GPIO` (nivel, voie de ieșire), pinii 0-31 vs 32-53
- [ ] **B2** — **butonul rotativ: intrare, întrerupere pe front, anti-zgomot** ← urmează
- [ ] **A4** — antet de imagine propriu, ca placa să pornească singură din flash
- [ ] **C** — alocator de memorie, card SD, FAT32
- [ ] **D** — planificator de sarcini, al doilea nucleu
- [ ] **E** — decodor MP3, I²S, sunet pe căști

### Ce deschide bătaia de 1000 Hz

Un planificator de sarcini *este* handlerul de bătaie, plus decizia de a te
întoarce în altă parte decât de unde ai plecat. Fundația de la etapa D e deja
turnată. Iar pe termen scurt, o apăsare de buton „țârâie” câteva milisecunde —
acum există cu ce s-o numeri.

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

Când raportul de excepție îți dă o adresă, bucla de depanare e:

```bash
xtensa-esp32s3-elf-objdump -d $ELF | grep -B8 -A2 "4037a018:"
```

Adresele și biții registrelor se scot din sursa descrisă a registrelor din cache-ul
cargo — folosită ca înlocuitor de manual, nu ca dependență: din ea copiem numere,
nu cod.

```bash
PAC=$(echo ~/.cargo/registry/src/index.crates.io-*/esp32s3-*/src)
grep -n -B4 'fn NUME(' $PAC/system.rs      # offsetul registrului
grep -n 'doc = "Bit'   $PAC/system/NUME.rs # câmpurile de biți
```
