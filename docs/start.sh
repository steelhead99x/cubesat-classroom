#!/usr/bin/env bash
set -euo pipefail

main() {
  local project cargo_bin cargo_home
  if [[ -f Cargo.toml && -f docs/start.sh && -d quest ]]; then
    project=$PWD
  elif [[ -f ${BASH_SOURCE[0]:-} ]]; then
    project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
  else
    project="$PWD/cubesat-classroom"
    if [[ ! -e "$project" ]]; then
      if ! command -v git >/dev/null; then
        printf 'Install Git, then run the starter again.\n' >&2
        return 1
      fi
      git clone --depth 1 https://github.com/steelhead99x/cubesat-classroom.git "$project"
    fi
  fi

  if [[ ! -f "$project/Cargo.toml" || ! -f "$project/docs/start.sh" || ! -d "$project/quest" ]]; then
    printf 'Choose a folder without an existing cubesat-classroom directory and try again.\n' >&2
    return 1
  fi

  cargo_home=${CARGO_HOME:-"$HOME/.cargo"}
  cargo_bin=$(command -v cargo || true)
  if [[ -z "$cargo_bin" ]]; then
    cargo_bin="$cargo_home/bin/cargo"
    if [[ ! -x "$cargo_bin" ]]; then
      printf 'Installing Rust with the official rustup installer.\n'
      local rust_installer
      rust_installer=$(curl -fsSL https://sh.rustup.rs)
      bash -s -- -y --profile minimal --no-modify-path <<<"$rust_installer"
    fi
  fi

  printf 'Project folder: %s\n' "$project"
  cd -- "$project"
  exec "$cargo_bin" run --locked -- classroom
}

main "$@"
