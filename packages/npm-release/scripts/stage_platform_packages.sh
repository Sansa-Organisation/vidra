#!/usr/bin/env bash
set -euo pipefail

# Stages native binaries into each platform npm package.
#
# Assumptions:
# - You already built archives under ./dist
# - Archives contain binaries at root (no subfolders)
#
# Customize these variables for your project.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TOOL_NAME="vidra"              # used in dist archive names
VERSION="0.0.0"                    # set to your release version

DIST_DIR="$ROOT_DIR/../dist"       # or "$ROOT_DIR/dist" depending on your repo

# Package directories (relative to this template)
PKG_DARWIN_ARM64="$ROOT_DIR/npm/platform-packages/darwin-arm64"
PKG_DARWIN_X64="$ROOT_DIR/npm/platform-packages/darwin-x64"
PKG_LINUX_X64="$ROOT_DIR/npm/platform-packages/linux-x64-musl"
PKG_LINUX_ARM64="$ROOT_DIR/npm/platform-packages/linux-arm64-musl"
PKG_WIN32_X64="$ROOT_DIR/npm/platform-packages/win32-x64-msvc"

need() { command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }; }
need tar
need unzip

stage_targz() {
  local target="$1"; shift
  local pkg_dir="$1"; shift
  local archive="$DIST_DIR/${TOOL_NAME}-v${VERSION}-${target}.tar.gz"
  [[ -f "$archive" ]] || { echo "ERROR: missing $archive" >&2; exit 1; }
  mkdir -p "$pkg_dir/bin"
  local tmp; tmp="$(mktemp -d)"
  tar xzf "$archive" -C "$tmp" >/dev/null
  # Adjust these filenames to your binaries:
  cp "$tmp/vidra" "$pkg_dir/bin/vidra"
  chmod 755 "$pkg_dir/bin/vidra"
  rm -rf "$tmp"
}

stage_zip_windows() {
  local target="$1"; shift
  local pkg_dir="$1"; shift
  local archive="$DIST_DIR/${TOOL_NAME}-v${VERSION}-${target}.zip"
  [[ -f "$archive" ]] || { echo "ERROR: missing $archive" >&2; exit 1; }
  mkdir -p "$pkg_dir/bin"
  local tmp; tmp="$(mktemp -d)"
  unzip -q "$archive" -d "$tmp"
  cp "$tmp/vidra.exe" "$pkg_dir/bin/vidra.exe"
  rm -rf "$tmp"
}

stage_targz aarch64-apple-darwin "$PKG_DARWIN_ARM64"
stage_targz x86_64-apple-darwin "$PKG_DARWIN_X64"
stage_targz x86_64-unknown-linux-musl "$PKG_LINUX_X64"
stage_targz aarch64-unknown-linux-musl "$PKG_LINUX_ARM64"
stage_zip_windows x86_64-pc-windows-msvc "$PKG_WIN32_X64"

echo "OK: staged platform packages" >&2
