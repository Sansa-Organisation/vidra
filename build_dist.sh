#!/bin/bash
set -e

mkdir -p dist
cd dist

# Copy the built binary
cp ../target/release/vidra ./vidra

# macOS ARM
tar -czvf vidra-v0.1.6-alpha.0-aarch64-apple-darwin.tar.gz vidra

# macOS x86 (dummy copy)
cp vidra-v0.1.6-alpha.0-aarch64-apple-darwin.tar.gz vidra-v0.1.6-alpha.0-x86_64-apple-darwin.tar.gz

# Linux x86 (dummy copy)
cp vidra-v0.1.6-alpha.0-aarch64-apple-darwin.tar.gz vidra-v0.1.6-alpha.0-x86_64-unknown-linux-musl.tar.gz

# Linux ARM (dummy copy)
cp vidra-v0.1.6-alpha.0-aarch64-apple-darwin.tar.gz vidra-v0.1.6-alpha.0-aarch64-unknown-linux-musl.tar.gz

# Windows x86 (dummy copy)
cp vidra vidra.exe
zip vidra-v0.1.6-alpha.0-x86_64-pc-windows-msvc.zip vidra.exe

echo "Dist archives created."
