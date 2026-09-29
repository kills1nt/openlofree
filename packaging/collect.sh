#!/usr/bin/env bash
# Gathers the built installers and flow2ctl into release-assets/ with uniform names.
# usage: packaging/collect.sh <platform-id> [rust-target]   e.g. linux-x64, macos-arm64 aarch64-apple-darwin
set -euo pipefail
id="$1"
target="${2:-}"
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/.*"version": "\(.*\)".*/\1/p' "$root/app/src-tauri/tauri.conf.json" | head -1)"
base="$root/target${target:+/$target}/release"
out="$root/release-assets"
mkdir -p "$out"

one() { # one <glob> <dest name>
  local f
  f="$(ls $1 2>/dev/null | head -1 || true)"
  [ -n "$f" ] || { echo "missing: $1" >&2; exit 1; }
  cp "$f" "$out/$2"
}

case "$id" in
  windows-*)
    one "$base/bundle/nsis/*-setup.exe" "openlofree-$version-$id-setup.exe"
    one "$base/bundle/msi/*.msi" "openlofree-$version-$id.msi"
    one "$base/openlofree-app.exe" "openlofree-$version-$id-portable.exe"
    one "$base/flow2ctl.exe" "flow2ctl-$version-$id.exe"
    ;;
  linux-*)
    one "$base/bundle/deb/*.deb" "openlofree-$version-$id.deb"
    one "$base/bundle/rpm/*.rpm" "openlofree-$version-$id.rpm"
    one "$base/bundle/appimage/*.AppImage" "openlofree-$version-$id.AppImage"
    cp "$root/packaging/linux/70-openlofree.rules" "$out/70-openlofree.rules"
    tar -czf "$out/flow2ctl-$version-$id.tar.gz" -C "$base" flow2ctl
    ;;
  macos-*)
    one "$base/bundle/dmg/*.dmg" "openlofree-$version-$id.dmg"
    tar -czf "$out/flow2ctl-$version-$id.tar.gz" -C "$base" flow2ctl
    ;;
  *) echo "unknown platform id: $id" >&2; exit 1 ;;
esac
ls -l "$out"
