#!/usr/bin/env bash
#
# Drive trippy-ui and capture a screenshot of every view, for development.
#
# It starts the binary, sends keys with xdotool and takes a screenshot with
# spectacle (KDE/Wayland) or import (X11) after each step, writing them to an
# output directory.
#
# Usage:
#   tools/screenshot.sh [--binary PATH] [--out DIR] [--] [TARGET...]
#
# Environment:
#   NETNS=1        run in a fresh loopback network namespace, so tracing works
#                  without CAP_NET_RAW (needs `unshare` and `ip`); only useful
#                  for exercising the UI, since only 127.0.0.1 is reachable.
#   DISPLAY=:0     the X11 display to drive; the app is forced onto X11 so that
#                  xdotool can send keys to it.
#
# On Linux without NETNS the binary needs `sudo setcap cap_net_raw+p`.
set -euo pipefail

BINARY="target/release/trippy-ui"
OUT="/tmp/trippy-ui-shots"
TARGETS=()

while [ $# -gt 0 ]; do
  case "$1" in
    --binary) BINARY="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --) shift; TARGETS=("$@"); break ;;
    *) TARGETS=("$@"); break ;;
  esac
done
[ ${#TARGETS[@]} -eq 0 ] && TARGETS=(127.0.0.1)

DISPLAY="${DISPLAY:-:0}"
export DISPLAY

for tool in xdotool; do
  command -v "$tool" >/dev/null || { echo "missing: $tool" >&2; exit 1; }
done
if command -v spectacle >/dev/null; then
  screenshot() { spectacle -b -n -f -o "$1" >/dev/null 2>&1; }
elif command -v import >/dev/null; then
  screenshot() { import -window root "$1" >/dev/null 2>&1; }
else
  echo "missing: spectacle or import" >&2; exit 1
fi

mkdir -p "$OUT"
rm -f "$OUT"/*.png

# The app is forced onto X11 so that xdotool can reach it; the winit variable
# wins over WAYLAND_DISPLAY, so unset the latter too.
run=(env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 "$BINARY" "${TARGETS[@]}")
if [ "${NETNS:-0}" = "1" ]; then
  run=(unshare -Urn sh -c "ip link set lo up; exec ${run[*]}")
fi

setsid "${run[@]}" >/tmp/trippy-ui-screenshot.log 2>&1 &
sleep 9

id=$(xdotool search --name trippy-ui | head -1)
[ -n "$id" ] || { echo "trippy-ui window not found" >&2; cat /tmp/trippy-ui-screenshot.log; exit 1; }
xdotool windowactivate --sync "$id" 2>/dev/null || true

shot() { sleep 0.6; screenshot "$OUT/$1.png"; echo "captured $1"; }
key() { xdotool key --clearmodifiers "$1" 2>/dev/null || true; }

shot 01-hops
key c; shot 02-chart
key m; shot 03-map
key m; shot 04-hops
key d; shot 05-details
key d
key s; shot 06-settings-tui
key 2; shot 07-settings-trace
key 7; shot 08-settings-columns
key 6; shot 09-settings-theme
key 5; shot 10-settings-bindings
key Escape
key h; shot 11-help
key Escape
key slash; xdotool type --clearmodifiers "1" 2>/dev/null || true; shot 12-filter
key Escape

pkill -x trippy-ui || true
sleep 1
echo "screenshots in $OUT"
