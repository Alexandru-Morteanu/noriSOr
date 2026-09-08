/* Harta memoriei — ESP32-S3, totul în RAM intern */

MEMORY
{
  iram : ORIGIN = 0x40378000, LENGTH = 128K
  dram : ORIGIN = 0x3FCA8000, LENGTH = 128K
}

ENTRY(_start)

SECTIONS
{
  .text : ALIGN(4)
  {
    /* Tabelul de constante al lui _start, apoi codul lui.
       Ordinea contează: l32r poate privi doar înapoi. */
    KEEP(*(.literal._start .text._start))

    /* Restul: pentru fiecare funcție, întâi constantele, apoi codul. */
    *(.literal .text .literal.* .text.*)
    . = ALIGN(4);
  } > iram

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*)
    . = ALIGN(4);
  } > dram

  .data : ALIGN(4)
  {
    *(.data .data.*)
    . = ALIGN(4);
  } > dram

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
