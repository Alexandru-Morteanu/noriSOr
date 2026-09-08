/* Tabela de vectori a ESP32-S3.
   Asamblata cu xtensa-esp32s3-elf-gcc, nu cu asamblorul din LLVM:
   s32e/l32e/rfwo/rfwu cer optiunea de ferestre, pe care doar el o are.
   Fiecare sectiune e plasata la offsetul ei fix de catre linker.ld. */

/* ---------- FERESTRE DE REGISTRE ----------
   Cand apelurile se cuibaresc destul cat sa se muste coada, procesorul
   ramane fara registre fizice si varsa automat fereastra cea mai veche
   pe stiva. s32e/l32e scriu si citesc in cadrul ferestrei VECINE —
   instructiuni care exista doar pentru asta. */

    .section .vector.wo4, "ax", @progbits
    .align 4
    .global _vec_wo4
_vec_wo4:
    s32e    a0, a5, -16
    s32e    a1, a5, -12
    s32e    a2, a5,  -8
    s32e    a3, a5,  -4
    rfwo

    .section .vector.wu4, "ax", @progbits
    .align 4
    .global _vec_wu4
_vec_wu4:
    l32e    a0, a5, -16
    l32e    a1, a5, -12
    l32e    a2, a5,  -8
    l32e    a3, a5,  -4
    rfwu

    .section .vector.wo8, "ax", @progbits
    .align 4
    .global _vec_wo8
_vec_wo8:
    s32e    a0, a9, -16
    l32e    a0, a1, -12
    s32e    a1, a9, -12
    s32e    a2, a9,  -8
    s32e    a3, a9,  -4
    s32e    a4, a0, -32
    s32e    a5, a0, -28
    s32e    a6, a0, -24
    s32e    a7, a0, -20
    rfwo

    .section .vector.wu8, "ax", @progbits
    .align 4
    .global _vec_wu8
_vec_wu8:
    l32e    a0, a9, -16
    l32e    a1, a9, -12
    l32e    a2, a9,  -8
    l32e    a7, a1, -12
    l32e    a3, a9,  -4
    l32e    a4, a7, -32
    l32e    a5, a7, -28
    l32e    a6, a7, -24
    l32e    a7, a7, -20
    rfwu

    .section .vector.wo12, "ax", @progbits
    .align 4
    .global _vec_wo12
_vec_wo12:
    s32e    a0,  a13, -16
    l32e    a0,  a1,  -12
    s32e    a1,  a13, -12
    s32e    a2,  a13,  -8
    s32e    a3,  a13,  -4
    s32e    a4,  a0,  -48
    s32e    a5,  a0,  -44
    s32e    a6,  a0,  -40
    s32e    a7,  a0,  -36
    s32e    a8,  a0,  -32
    s32e    a9,  a0,  -28
    s32e    a10, a0,  -24
    s32e    a11, a0,  -20
    rfwo

    .section .vector.wu12, "ax", @progbits
    .align 4
    .global _vec_wu12
_vec_wu12:
    l32e    a0,  a13, -16
    l32e    a1,  a13, -12
    l32e    a2,  a13,  -8
    l32e    a11, a1,  -12
    l32e    a3,  a13,  -4
    l32e    a4,  a11, -48
    l32e    a5,  a11, -44
    l32e    a6,  a11, -40
    l32e    a7,  a11, -36
    l32e    a8,  a11, -32
    l32e    a9,  a11, -28
    l32e    a10, a11, -24
    l32e    a11, a11, -20
    rfwu

/* ---------- INTRERUPERI PE NIVELE ---------- */

    .section .vector.lvl2, "ax", @progbits
    .align 4
    .global _vec_lvl2
_vec_lvl2:   j _vec_lvl2

    .section .vector.lvl3, "ax", @progbits
    .align 4
    .global _vec_lvl3
_vec_lvl3:   j _vec_lvl3

    .section .vector.lvl4, "ax", @progbits
    .align 4
    .global _vec_lvl4
_vec_lvl4:   j _vec_lvl4

    .section .vector.lvl5, "ax", @progbits
    .align 4
    .global _vec_lvl5
_vec_lvl5:   j _vec_lvl5

    .section .vector.dbg, "ax", @progbits
    .align 4
    .global _vec_dbg
_vec_dbg:    j _vec_dbg

    .section .vector.nmi, "ax", @progbits
    .align 4
    .global _vec_nmi
_vec_nmi:    j _vec_nmi

/* ---------- EXCEPTII (se umplu la pasul 4) ---------- */

    .section .vector.kernel, "ax", @progbits
    .align 4
    .global _vec_kernel
_vec_kernel: j _vec_kernel

    .section .vector.user, "ax", @progbits
    .align 4
    .global _vec_user
_vec_user:
    wsr.excsave1 a0
    rsr.exccause a0
    bnei    a0, 4, _exc_lent
    j       _int_tick
_exc_lent:
    movi    a0, 1
    slli    a0, a0, 18
    addi    a0, a0, 0x2F
    wsr.ps  a0
    rsync
    j       _exc_report

    .section .vector.double, "ax", @progbits
    .align 4
    .global _vec_double
_vec_double: j _vec_double


/* ---------- BATAIA DE INIMA ----------
   Rulam aici cu EXCM inca pe 1: nicio alta intrerupere, nicio exceptie
   de fereastra. Deci nu avem voie sa chemam nicio functie Rust (ar incepe
   cu entry, care roteste fereastra). Atingem trei registre si le punem
   inapoi. Suntem in .text, deci l32r are voie sa priveasca inapoi. */

    .section .text.int_tick, "ax", @progbits
    .literal .Lpas,  240000        /* 240 MHz / 1000 = o bataie pe milisecunda */
    .literal .Ltick, TICKS         /* adresa contorului din Rust */
    .align 4
    .global _int_tick
_int_tick:
    addi    a1, a1, -32            /* cadru propriu, sub cel intrerupt */
    s32i    a2, a1, 16
    s32i    a3, a1, 20

    rsr.ccompare0 a2               /* pragul urmator = pragul vechi + pas.  */
    l32r    a3, .Lpas              /* scrierea in CCOMPARE0 sterge si       */
    add     a2, a2, a3             /* cererea de intrerupere.               */
    wsr.ccompare0 a2

    l32r    a3, .Ltick             /* TICKS += 1 */
    l32i    a2, a3, 0
    addi    a2, a2, 1
    s32i    a2, a3, 0

    l32i    a2, a1, 16             /* punem totul exact cum era */
    l32i    a3, a1, 20
    addi    a1, a1, 32
    rsr.excsave1 a0
    rfe                            /* inapoi la instructiunea intrerupta */
