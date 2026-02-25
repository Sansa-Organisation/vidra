#!/usr/bin/env bash

set -euo pipefail

echo "[vidra] local quality gate"

if [[ "${VIDRA_STRICT_FMT:-0}" == "1" ]]; then
	echo "[1/4] format check"
	cargo fmt --all -- --check
else
	echo "[1/4] format check (skipped; set VIDRA_STRICT_FMT=1 to enable)"
fi

echo "[2/4] strict compile (warnings denied)"
RUSTFLAGS="-D warnings" cargo check --workspace --all-targets

echo "[3/4] test suite"
cargo test --workspace

echo "[4/4] MCP stdio purity"
cargo test -p vidra-cli --test mcp_stdio_purity

echo "[vidra] local quality gate passed"
