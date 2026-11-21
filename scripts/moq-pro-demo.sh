#!/usr/bin/env bash
set +x
set -euo pipefail

mode=${1:-}
project=${2:-}
broadcast='cubesat-classroom-test.hang'

usage() {
  printf 'Usage: bash scripts/moq-pro-demo.sh {publish|watch|check} YOUR_PRIVATE_PROJECT\n'
  printf 'Use a short-lived credential for the selected role. The demo uses generated video.\n'
}

case "$mode" in
  publish|watch|check) ;;
  -h|--help|'') usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac

if [[ ! "$project" =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]*$ || "$project" =~ ^([Aa][Nn][Oo][Nn]|[Dd][Ee][Mm][Oo]|[Tt][Rr][Yy]|[Pp][Uu][Bb][Ll][Ii][Cc])$ ]]; then
  printf 'Enter the ID of your own private MoQ.pro project. Shared project names are disabled.\n' >&2
  exit 2
fi

if [[ "$mode" == watch ]]; then
  player=ffplay
else
  player=ffmpeg
fi
for command in moq "$player"; do
  if ! command -v "$command" >/dev/null; then
    printf 'Install %s before running this demo. See docs/moq-pro-development.md.\n' "$command" >&2
    exit 2
  fi
done

moq_help=$(moq --help)
if [[ "$moq_help" != *'env: MOQ_CONNECT'* ]]; then
  printf 'Install a MoQ CLI release with MOQ_CONNECT support.\n' >&2
  exit 2
fi

if [[ "$mode" == publish ]]; then
  role=publisher
else
  role=subscriber
fi

if ! { exec 3<>/dev/tty; } 2>/dev/null; then
  printf 'Open a terminal to enter the credential without echoing it.\n' >&2
  exit 2
fi
printf 'Paste the short-lived %s token for %s (input hidden): ' "$role" "$project" >&3
IFS= read -rs token <&3 || exit 2
printf '\n' >&3
exec 3>&-
if [[ ! "$token" =~ ^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$ ]]; then
  printf 'Use a project-scoped stream JWT from your MoQ provider, not an account API key.\n' >&2
  exit 2
fi

# Keep the token out of arguments, shell history, ffmpeg, and client logs.
run_moq() (
  for setting in ${!MOQ_@}; do
    unset "$setting"
  done
  export MOQ_LOG_LEVEL=off
  export MOQ_CONNECT="https://cdn.moq.pro/$project"
  if [[ "$moq_help" == *MOQ_CONNECT_TOKEN* ]]; then
    export MOQ_CONNECT_TOKEN="$token"
  else
    MOQ_CONNECT="$MOQ_CONNECT?jwt=$token"
  fi
  exec moq --connect-once --connect-timeout 15s --broadcast "$broadcast" "$@" 2>/dev/null
)

case "$mode" in
  publish)
    printf 'Publishing a 45-second generated test pattern. Start the subscriber in another terminal.\n'
    if ! ffmpeg -hide_banner -loglevel error -nostdin -re -f lavfi -i 'testsrc2=size=640x360:rate=15' \
      -t 45 -c:v libx264 -preset ultrafast -tune zerolatency -g 15 \
      -movflags cmaf+separate_moof+delay_moov+skip_trailer -frag_duration 1000000 -f mp4 pipe:1 |
      run_moq import fmp4; then
      printf 'Publishing failed. Check the project, publish grant, and relay reachability.\n' >&2
      exit 1
    fi
    ;;
  watch)
    if ! run_moq export fmp4 | ffplay -hide_banner -loglevel error -autoexit -i pipe:0; then
      printf 'Playback ended or failed. Check the subscribe grant and the publisher.\n' >&2
      exit 1
    fi
    ;;
  check)
    # The decoder stops after 15 frames, which can end the upstream pipe early.
    if run_moq export fmp4 | ffmpeg -hide_banner -loglevel error -nostdin \
        -i pipe:0 -map 0:v:0 -frames:v 15 -abort_on empty_output -f null -; then
      printf 'Decoded the private test stream.\n'
    elif [[ ${PIPESTATUS[1]} == 0 ]]; then
      printf 'Decoded the private test stream.\n'
    else
      printf 'No video decoded. Check the subscribe grant and the publisher.\n' >&2
      exit 1
    fi
    ;;
esac
