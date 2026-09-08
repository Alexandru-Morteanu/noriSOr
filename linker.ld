/* Harta memoriei — ESP32-S3, totul în RAM intern */

MEMORY
{
  iram : ORIGIN = 0x40378000, LENGTH = 128K
  dram : ORIGIN = 0x3FCA8000, LENGTH = 128K
}

ENTRY(_start)

SECTIONS
{
  /* TABELA DE VECTORI.
     Aliniată la 1 KB, pentru că VECBASE nu poate arăta decât la un
     multiplu de 0x400. În interior, fiecare vector are un loc impus de
     hardware — nu le alegem noi, le nimerim. */
  .vectors : ALIGN(1024)
  {
    _vecbase = ABSOLUTE(.);
                 KEEP(*(.vector.wo4))     /* +0x000 ferestre */
    . = 0x040;   KEEP(*(.vector.wu4))
    . = 0x080;   KEEP(*(.vector.wo8))
    . = 0x0C0;   KEEP(*(.vector.wu8))
    . = 0x100;   KEEP(*(.vector.wo12))
    . = 0x140;   KEEP(*(.vector.wu12))
    . = 0x180;   KEEP(*(.vector.lvl2))    /* +0x180 intreruperi */
    . = 0x1C0;   KEEP(*(.vector.lvl3))
    . = 0x200;   KEEP(*(.vector.lvl4))
    . = 0x240;   KEEP(*(.vector.lvl5))
    . = 0x280;   KEEP(*(.vector.dbg))
    . = 0x2C0;   KEEP(*(.vector.nmi))
    . = 0x300;   KEEP(*(.vector.kernel))  /* +0x300 exceptii */
    . = 0x340;   KEEP(*(.vector.user))
    . = 0x3C0;   KEEP(*(.vector.double))
    . = 0x400;
  } > iram

  .text : ALIGN(4)
  {
    /* Tabelul de constante al lui _start, apoi codul lui.
       Ordinea contează: l32r poate privi doar înapoi. */
    KEEP(*(.literal._start .text._start))
    *(.literal .text .literal.* .text.*)
    . = ALIGN(4);
  } > iram

  .rodata : ALIGN(4) { *(.rodata .rodata.*) . = ALIGN(4); } > dram
  .data   : ALIGN(4) { *(.data .data.*)     . = ALIGN(4); } > dram

  .bss (NOLOAD) : ALIGN(4)
  {
    _bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(4);
    _bss_end = .;
  } > dram

  _stack_top = ORIGIN(dram) + LENGTH(dram);

  /DISCARD/ : { *(.eh_frame) *(.eh_frame_hdr) }
}
