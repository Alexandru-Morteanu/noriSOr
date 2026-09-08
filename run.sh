#!/bin/bash
set -e
cd ~/Desktop/player
ELF=~/.cargo-target/player/xtensa-esp32s3-none-elf/debug/player

# 1. eliberam cablul de orice a ramas agatat de el
pkill -9 -f "cat /dev/cu.usbmodem" 2>/dev/null || true
pkill -9 screen  2>/dev/null || true
pkill -9 openocd 2>/dev/null || true
sleep 1

# 2. compilam
cargo build

# 3. aflam singuri punctul de intrare din ELF (se muta cand creste codul)
ENTRY=$(xtensa-esp32s3-elf-readelf -h "$ELF" | awk '/Entry point/ {print $4}')
echo ">>> punct de intrare: $ENTRY"

# 4. incarcam in RAM prin JTAG, punem degetul pe _start, dam drumul, plecam
openocd -f board/esp32s3-builtin.cfg \
  -c "init" -c "reset halt" \
  -c "load_image $ELF" \
  -c "reg pc $ENTRY" \
  -c "resume" \
  -c "shutdown"

# 5. asteptam sa reapara portul (se renumera dupa ce pleaca OpenOCD)
PORT=""
for i in $(seq 1 20); do
  PORT=$(ls /dev/cu.usbmodem* 2>/dev/null | head -1)
  [ -n "$PORT" ] && break
  sleep 0.5
done
[ -z "$PORT" ] && { echo "!!! nu gasesc portul USB"; exit 1; }

# 6. ascultam ce spune chipul
echo ">>> ascult pe $PORT   (Ctrl-C ca sa iesi)"
cat "$PORT"
