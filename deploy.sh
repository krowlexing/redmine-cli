#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n1)"
[ -n "$VERSION" ] || { echo "error: cannot read version from Cargo.toml" >&2; exit 1; }

LINUX_BIN="target/release/redmine"
WIN_TARGET="x86_64-pc-windows-gnu"
WIN_BIN="target/$WIN_TARGET/release/redmine.exe"
DIST="dist"

MODE="${1:-all}"
case "$MODE" in
  linux | windows | all) ;;
  *)
    echo "usage: $0 [linux|windows|all]" >&2
    exit 2
    ;;
esac

artifact() {
  local src="$1" name="$2"
  mkdir -p "$DIST"
  cp "$src" "$DIST/$name"
  sha256sum "$DIST/$name" >"$DIST/$name.sha256"
  ls -lh "$DIST/$name" | awk '{print "==> artifact", $5, $9}'
}

check_linux() {
  command -v cargo >/dev/null || {
    echo "error: cargo not found" >&2
    FAIL=1
  }
  command -v strip >/dev/null || {
    echo "error: strip not found" >&2
    FAIL=1
  }
}

check_windows() {
  if ! command -v rustup >/dev/null || ! rustup target list --installed 2>/dev/null | grep -qx "$WIN_TARGET"; then
    echo "error: rust target not installed; run: rustup target add $WIN_TARGET" >&2
    FAIL=1
  fi
  if ! command -v x86_64-w64-mingw32-gcc >/dev/null; then
    echo "error: mingw-w64 linker not installed; run: apt install mingw-w64" >&2
    FAIL=1
  fi
}

build_linux() {
  check_linux
  [ "${FAIL:-0}" -eq 0 ] || exit 1
  echo "==> building linux release ($VERSION)"
  cargo build --release
  echo "==> unstripped: $(du -h "$LINUX_BIN" | cut -f1)"
  strip "$LINUX_BIN"
  echo "==> stripped:   $(du -h "$LINUX_BIN" | cut -f1)"
  artifact "$LINUX_BIN" "redmine"
}

build_windows() {
  check_windows
  [ "${FAIL:-0}" -eq 0 ] || exit 1
  echo "==> building windows release ($VERSION)"
  cargo build --release --target "$WIN_TARGET"
  if command -v x86_64-w64-mingw32-strip >/dev/null; then
    x86_64-w64-mingw32-strip "$WIN_BIN"
  else
    echo "warning: x86_64-w64-mingw32-strip not found, shipping unstripped exe" >&2
  fi
  artifact "$WIN_BIN" "redmine.exe"
}

case "$MODE" in
linux) build_linux ;;
windows) build_windows ;;
all)
  FAIL=0
  check_linux
  check_windows
  if [ "$FAIL" -ne 0 ]; then
    echo "error: toolchains missing for default build; run './deploy.sh linux' or './deploy.sh windows' to build selectively" >&2
    exit 1
  fi
  build_linux
  build_windows
  ;;
esac

echo "==> done, artifacts in $DIST/"
