#!/usr/bin/env bash
# Compiles every firmware target, the same set GitHub CI's firmware jobs build.
#
#   bash scripts/firmware.sh            Rust Nano firmware, then every Arduino board
#   bash scripts/firmware.sh --rust     only the Rust Nano 33 BLE Sense firmware
#   bash scripts/firmware.sh --arduino [FQBN ...]   only the Arduino sketches (all boards,
#                                       or the boards named)
#
# A missing toolchain is an error, never a quiet skip.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

TARGET=thumbv7em-none-eabihf
ESP32_URL=https://espressif.github.io/arduino-esp32/package_esp32_index.json
RP2040_URL=https://github.com/earlephilhower/arduino-pico/releases/download/global/package_rp2040_index.json
SEEED_URL=https://files.seeedstudio.com/arduino/package_seeeduino_boards_index.json
SILABS_URL=https://siliconlabs.github.io/arduino/package_arduinosilabs_index.json
XIAO_LIBS=("Adafruit BME280 Library" "Adafruit TSL2561" "Adafruit Unified Sensor" "Adafruit BusIO" "U8g2")

# One line per board: fqbn|core|board-manager URL|sketches (comma separated).
BOARDS=(
  "arduino:mbed_nano:nano33ble|arduino:mbed_nano||firmware/arduino-cubesat"
  "arduino:avr:uno|arduino:avr||firmware/classroom-sensors,hardware/sdr-receiver/bench/power-logger"
  "esp32:esp32:XIAO_ESP32S3|esp32:esp32|$ESP32_URL|firmware/xiao-cubesat,firmware/classroom-sensors,hardware/rideshare-flatsat/s3_recorder,hardware/sdr-receiver/bench/power-logger"
  "esp32:esp32:XIAO_ESP32C3|esp32:esp32|$ESP32_URL|firmware/xiao-cubesat,firmware/classroom-sensors,hardware/sdr-receiver/bench/power-logger"
  "rp2040:rp2040:seeed_xiao_rp2040|rp2040:rp2040|$RP2040_URL|firmware/xiao-cubesat,firmware/classroom-sensors,hardware/sdr-receiver/bench/power-logger"
  "Seeeduino:samd:seeed_XIAO_m0|Seeeduino:samd|$SEEED_URL|firmware/xiao-cubesat,firmware/classroom-sensors"
  # Rideshare FlatSat: the MG24 Sense IMU sender (its S3 recorder builds with the S3 line above).
  "SiliconLabs:silabs:xiao_mg24|SiliconLabs:silabs|$SILABS_URL|hardware/rideshare-flatsat/mg24_sensor"
)

libs_for() {
  case "$1" in
    arduino:mbed_nano:*) printf '%s\n' Arduino_APDS9960 Arduino_BMI270_BMM150 Arduino_HS300x Arduino_LPS22HB ;;
    arduino:avr:*) ;;
    SiliconLabs:silabs:*) printf '%s\n' "Seeed Arduino LSM6DS3" ;;
    Seeeduino:samd:*) printf '%s\n' "${XIAO_LIBS[@]}" SD ;;
    *) printf '%s\n' "${XIAO_LIBS[@]}" ;;
  esac
}

die() { echo "firmware.sh: $*" >&2; exit 1; }

rust() {
  echo "=== Nano 33 BLE Sense firmware (Rust, $TARGET) ==="
  command -v cargo >/dev/null 2>&1 || die "cargo is not installed"
  if command -v rustup >/dev/null 2>&1; then
    rustup target list --installed | grep -qx "$TARGET" \
      || die "Rust target $TARGET is missing; run: rustup target add $TARGET"
  fi
  (
    cd firmware/nano33-cubesat
    cargo fmt --check
    cargo clippy --locked --release -- -D warnings
    cargo build --locked --release
  )
}

arduino() {
  command -v arduino-cli >/dev/null 2>&1 || die "arduino-cli is not installed (https://arduino.github.io/arduino-cli/)"
  local wanted=("$@") found=0 line fqbn core url sketches
  for line in "${BOARDS[@]}"; do
    IFS='|' read -r fqbn core url sketches <<< "$line"
    if [ ${#wanted[@]} -gt 0 ] && ! printf '%s\n' "${wanted[@]}" | grep -qx "$fqbn"; then
      continue
    fi
    found=$((found + 1))
    echo "=== Arduino $fqbn ==="
    local urls=()
    [ -n "$url" ] && urls=(--additional-urls "$url")
    arduino-cli core update-index "${urls[@]}"
    arduino-cli core install "$core" "${urls[@]}"
    local lib
    while IFS= read -r lib; do
      [ -n "$lib" ] && arduino-cli lib install "$lib"
    done < <(libs_for "$fqbn")
    local sketch
    IFS=',' read -r -a sketch_list <<< "$sketches"
    for sketch in "${sketch_list[@]}"; do
      arduino-cli compile --fqbn "$fqbn" "${urls[@]}" "$sketch"
    done
  done
  if [ ${#wanted[@]} -gt 0 ] && [ "$found" -ne ${#wanted[@]} ]; then
    die "unknown board in: ${wanted[*]}"
  fi
}

case "${1:-}" in
  --rust) rust ;;
  --arduino) shift; arduino "$@" ;;
  "") rust; arduino ;;
  *) die "usage: firmware.sh [--rust | --arduino [FQBN ...]]" ;;
esac
