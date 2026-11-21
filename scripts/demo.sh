#!/usr/bin/env bash
# Software-only rehearsal. Never opens a radio or camera device.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
if [[ $# != 1 ]]; then
  echo "Usage: $0 /path/to/moq-subghz" >&2
  exit 2
fi
subghz=$(cd -- "$1" && pwd)
cargo build --locked
cargo build --locked --manifest-path "$subghz/Cargo.toml" -p moq-subghz-send -p moq-subghz-recv
cargo build --locked --manifest-path "$subghz/moq-subghz-publish/Cargo.toml"

mkdir -p run
work=$(mktemp -d "$PWD/run/rehearsal.XXXXXX")
echo "Evidence directory: $work"
pids=()
cleanup() {
  local pid
  for pid in "${pids[@]}"; do kill "$pid" 2>/dev/null || true; done
  wait || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

app="$PWD/target/debug/cubesat-classroom"
"$app" relay --port 0 >"$work/relay.txt" &
pids+=("$!")
unset CUBESAT_RELAY CUBESAT_TLS_FINGERPRINT
for _ in {1..50}; do
  if [[ -s "$work/relay.txt" ]] && read -r CUBESAT_RELAY CUBESAT_TLS_FINGERPRINT <"$work/relay.txt"; then break; fi
  sleep .1
done
: "${CUBESAT_RELAY:?Relay did not start}"
: "${CUBESAT_TLS_FINGERPRINT:?Relay fingerprint missing}"
export CUBESAT_RELAY CUBESAT_TLS_FINGERPRINT
export MOQ_SUBGHZ_TLS_FINGERPRINT="$CUBESAT_TLS_FINGERPRINT"

"$app" simulate &
pids+=("$!")
"$app" read-telemetry --reading status | tee "$work/telemetry.txt"

"$subghz/target/debug/moq-subghz-send" --loopback "$work/air.txt" --fec-k 8 --secs 8 >"$work/send.txt" 2>"$work/send.stderr"
"$subghz/target/debug/moq-subghz-recv" --replay "$work/air.txt" --paced --decode --out "$work/receiver" \
  --publish "$CUBESAT_RELAY" --publish-name cubesat/classroom/video \
  --publish-bin "$subghz/moq-subghz-publish/target/debug/moq-subghz-publish" \
  >"$work/receive.txt" 2>"$work/receive.stderr" &
receiver=$!
pids+=("$receiver")
"$app" verify-video --broadcast cubesat/classroom/video | tee "$work/video.txt"
"$app" verify-video --broadcast cubesat/classroom/video --stream >"$work/quic.h264" 2>"$work/quic.stderr"
wait "$receiver"
ffmpeg -nostdin -v error -xerror -f h264 -i "$work/quic.h264" -f null -
test -s "$work/receiver/rec.h264"
