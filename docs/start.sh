#!/usr/bin/env bash
set -euo pipefail

ask() {
  local answer
  if [[ "$assume_yes" == true ]]; then
    return 0
  fi

  while true; do
    printf '%s [Y/n] ' "$1" >&3
    if ! IFS= read -r answer <&3; then
      printf '\nSetup stopped. Run the same command to try again.\n'
      return 1
    fi

    case "$answer" in
      ''|[Yy]|[Yy][Ee][Ss]) return 0 ;;
      [Nn]|[Nn][Oo])
        printf 'Setup stopped. Run the same command to pick up again.\n'
        return 1
        ;;
      *) printf 'Press Enter to continue, or type n to stop.\n' >&3 ;;
    esac
  done
}

choose_quest() {
  quest_path=quest/m0/repeat-the-station-lesson.md
  if [[ "$assume_yes" == true ]]; then
    return
  fi

  printf '\nChoose a first quest based on the hardware you have.\n'
  printf '  1. Computer only or unsure (default)\n'
  printf '  2. Arduino, for example an Uno\n'
  printf '  3. Seeed XIAO, for example ESP32-C3 or RP2040\n'
  printf '  4. Raspberry Pi running Raspberry Pi OS\n'
  printf '  5. Pico or another board\n'
  local choice
  while true; do
    printf 'Your setup [1–5]: ' >&3
    IFS= read -r choice <&3 || return 1
    case "$choice" in
      ''|1) break ;;
      2) quest_path=quest/m0/arduino-first-reading.md; break ;;
      3) quest_path=quest/m0/xiao-first-reading.md; break ;;
      4) quest_path=quest/m0/pi-first-reading.md; break ;;
      5) quest_path=quest/README.md; break ;;
      *) printf 'Choose a number from 1 to 5.\n' >&3 ;;
    esac
  done
}

main() {
  local project cargo_bin cargo_home rustup_bin quest_path
  local assume_yes=false

  case "${1:-}" in
    '') ;;
    --yes) assume_yes=true ;;
    -h|--help)
      printf 'Usage: bash docs/start.sh [--yes]\nUse --yes for unattended setup.\n'
      return 0
      ;;
    *) printf 'Unknown option: %s\n' "$1" >&2; return 2 ;;
  esac

  if [[ "$assume_yes" == false ]] && ! { exec 3<>/dev/tty; } 2>/dev/null; then
    printf 'Open a terminal to follow the guided setup. For an unattended run, pipe to bash -s -- --yes.\n' >&2
    return 2
  fi

  printf '\nCubeSat Classroom\n'
  printf 'A CubeSat is a small satellite. A ground station sends it requests and receives data.\n'
  printf 'Today you will practice on a simulated satellite running on your computer.\n'
  choose_quest || return 0

  printf '\n1. Get the classroom files\n'
  printf 'The files include the simulator, lessons, and quests you can share with the next crew.\n'
  if [[ -f Cargo.toml && -f docs/start.sh && -d quest ]]; then
    project=$PWD
  elif [[ -f ${BASH_SOURCE[0]:-} ]]; then
    project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
  else
    project="$PWD/cubesat-classroom"
    if [[ ! -e "$project" ]]; then
      if ! command -v git >/dev/null; then
        printf 'Install Git with help from a teacher or mentor, then run this command again.\n' >&2
        return 1
      fi
      ask "Download the open-source files to $project?" || return 0
      git clone --depth 1 https://github.com/steelhead99x/cubesat-classroom.git "$project"
    fi
  fi

  if [[ ! -f "$project/Cargo.toml" || ! -f "$project/docs/start.sh" || ! -d "$project/quest" ]]; then
    printf 'Choose a folder without an existing cubesat-classroom directory and try again.\n' >&2
    return 1
  fi
  printf 'Classroom files: %s\n' "$project"
  printf 'Your recommended first quest: %s/%s\n' "$project" "$quest_path"

  printf '\n2. Check the build tools\n'
  printf 'The simulator is written in Rust. Cargo builds and starts it.\n'
  cargo_home=${CARGO_HOME:-"$HOME/.cargo"}
  cargo_bin=$(command -v cargo || true)
  cargo_bin=${cargo_bin:-"$cargo_home/bin/cargo"}
  rustup_bin=$(command -v rustup || true)
  rustup_bin=${rustup_bin:-"$cargo_home/bin/rustup"}
  if [[ ! -x "$cargo_bin" ]]; then
    ask 'Install Rust and Cargo with the official Rust installer?' || return 0
    local rust_installer
    rust_installer=$(curl -fsSL https://sh.rustup.rs)
    bash -s -- -y --profile minimal --no-modify-path <<<"$rust_installer"
  elif [[ -x "$rustup_bin" ]] && ! "$rustup_bin" show active-toolchain >/dev/null 2>&1; then
    ask 'Download the stable Rust toolchain with rustup?' || return 0
    "$rustup_bin" default stable
  else
    printf 'Cargo is ready.\n'
  fi

  printf '\n3. Start the practice ground station\n'
  printf 'The first build downloads dependencies and may take a few minutes.\n'
  ask 'Build and launch the station?' || return 0
  printf '\nOpen the local address the station prints after the build.\n'
  printf 'Your first mission: request a sensor reading, write it down with its unit, then try the practice video.\n'
  printf 'When you are done, press Ctrl+C here and open your recommended quest.\n\n'

  cd -- "$project"
  if [[ "$assume_yes" == false ]]; then
    exec 3>&-
  fi
  exec "$cargo_bin" run --locked -- classroom
}

main "$@"
