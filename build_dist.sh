#!/bin/bash
set -e

VERSION="0.1.7-alpha.0"

echo "🔧 Building Vidra CLI for all platforms..."
echo ""

mkdir -p dist
rm -f dist/vidra-v${VERSION}-*.tar.gz dist/vidra-v${VERSION}-*.zip

# ── macOS ARM64 (native) ─────────────────────────────────────────────
echo "  [1/4] aarch64-apple-darwin (native)"
cargo build --release -p vidra-cli --target aarch64-apple-darwin 2>/dev/null
(cd target/aarch64-apple-darwin/release && tar -czf ../../../dist/vidra-v${VERSION}-aarch64-apple-darwin.tar.gz vidra)
echo "        ✓ done"

# ── macOS x86_64 ─────────────────────────────────────────────────────
echo "  [2/4] x86_64-apple-darwin"
cargo build --release -p vidra-cli --target x86_64-apple-darwin 2>/dev/null
(cd target/x86_64-apple-darwin/release && tar -czf ../../../dist/vidra-v${VERSION}-x86_64-apple-darwin.tar.gz vidra)
echo "        ✓ done"

# ── Linux x86_64 (musl — static, via zig) ───────────────────────────
echo "  [3/4] x86_64-unknown-linux-musl (via cargo-zigbuild)"
cargo zigbuild --release -p vidra-cli --target x86_64-unknown-linux-musl 2>/dev/null
(cd target/x86_64-unknown-linux-musl/release && tar -czf ../../../dist/vidra-v${VERSION}-x86_64-unknown-linux-musl.tar.gz vidra)
echo "        ✓ done"

# ── Linux ARM64 (musl — static, via zig) ─────────────────────────────
echo "  [4/4] aarch64-unknown-linux-musl (via cargo-zigbuild)"
cargo zigbuild --release -p vidra-cli --target aarch64-unknown-linux-musl 2>/dev/null
(cd target/aarch64-unknown-linux-musl/release && tar -czf ../../../dist/vidra-v${VERSION}-aarch64-unknown-linux-musl.tar.gz vidra)
echo "        ✓ done"

# ── Windows: skip for now (requires MSVC linker, not available on macOS)
# For Windows builds, use GitHub Actions CI or a Windows machine.
echo ""
echo "  ⚠  Windows (x86_64-pc-windows-msvc) skipped — requires CI or Windows machine"

echo ""
echo "📦 Archives:"
ls -lh dist/vidra-v${VERSION}-*
echo ""
echo "✅ All available platforms built."
