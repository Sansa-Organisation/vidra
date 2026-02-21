#!/usr/bin/env bash
# Vidra — Install Script
# Usage: curl -fsSL https://vidra.dev/install.sh | sh
#
# Installs the latest version of the Vidra CLI.

set -euo pipefail

REPO="vidra-dev/vidra"
INSTALL_DIR="${VIDRA_INSTALL_DIR:-$HOME/.vidra/bin}"

# ── Detect Platform ──────────────────────────────────────────────────

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)   os="linux" ;;
        Darwin*)  os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)        echo "❌ Unsupported OS: $(uname -s)" && exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)             echo "❌ Unsupported architecture: $(uname -m)" && exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# ── Fetch Latest Version ─────────────────────────────────────────────

get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"v([^"]+)".*/\1/'
}

# ── Install ──────────────────────────────────────────────────────────

main() {
    echo "🎬 Vidra Installer"
    echo ""

    local platform version archive_name url

    platform=$(detect_platform)
    echo "   Platform: ${platform}"

    version=$(get_latest_version 2>/dev/null || echo "0.1.0")
    echo "   Version:  v${version}"
    echo ""

    archive_name="vidra-v${version}-${platform}.tar.gz"
    url="https://github.com/${REPO}/releases/download/v${version}/${archive_name}"

    echo "⬇️  Downloading ${archive_name}..."
    local tmp_dir
    tmp_dir=$(mktemp -d)
    curl -fsSL "${url}" -o "${tmp_dir}/${archive_name}" 2>/dev/null || {
        echo "⚠️  Download failed. Installing from source via cargo..."
        if command -v cargo &>/dev/null; then
            cargo install --git "https://github.com/${REPO}" vidra-cli
            echo "✅ Vidra installed via cargo!"
            return
        else
            echo "❌ Neither a prebuilt binary nor cargo is available."
            exit 1
        fi
    }

    echo "📦 Extracting..."
    mkdir -p "${INSTALL_DIR}"
    tar -xzf "${tmp_dir}/${archive_name}" -C "${INSTALL_DIR}"
    chmod +x "${INSTALL_DIR}/vidra"
    rm -rf "${tmp_dir}"

    echo ""
    echo "✅ Vidra v${version} installed to ${INSTALL_DIR}/vidra"
    echo ""

    # Check if INSTALL_DIR is in PATH
    if ! echo "$PATH" | tr ':' '\n' | grep -q "^${INSTALL_DIR}$"; then
        echo "⚠️  ${INSTALL_DIR} is not in your PATH."
        echo "   Add this to your shell profile:"
        echo ""
        echo "   export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
    fi

    echo "   Run 'vidra --help' to get started."
    echo "   Run 'vidra init my-project' to create your first project."
    echo ""
}

main "$@"
