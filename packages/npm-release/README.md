# OptionalDependencies Native-Binary Release Template

This template shows the **multi-package npm distribution** pattern for shipping native binaries via per-platform `optionalDependencies`.

## What you get

- 1 main “wrapper” package users install/run (e.g. `@scope/tool`)
- N platform packages containing the native binaries (e.g. `@scope/tool-darwin-arm64`)
- A staging script that copies built binaries into the platform packages before publishing

## Why this pattern

- No `postinstall` downloader script.
- Package managers automatically install only the matching platform optionalDependency.
- Clean installs (works better in stricter environments and under Bun/bunx).

## How to adapt

1) Pick your names

- `SCOPE` (e.g. `@acme`)
- `MAIN_PKG` (e.g. `@acme/tool`)
- `PLATFORM_PKGS` (e.g. `@acme/tool-darwin-arm64`, etc.)

2) Update `npm/main-wrapper/package.json`

- Set `name`, `version`
- Edit `optionalDependencies` to match your platform packages

3) Update the platform package manifests

Each platform package in `npm/platform-packages/*/package.json` must set:
- `name`, `version`
- `os` and `cpu`
- ship `bin/` directory containing your native executables

4) Build your native binaries and create archives in `dist/`

This template assumes you produce archives like:

- `dist/TOOL-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- `dist/TOOL-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `dist/TOOL-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`
- `dist/TOOL-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz`
- `dist/TOOL-vX.Y.Z-x86_64-pc-windows-msvc.zip`

…and that archives contain binaries at the root (e.g. `tool`, `tool-mcp`, or `tool.exe`).

5) Stage + publish

- Run the staging script: `scripts/stage_platform_packages.sh`
- Publish platform packages first
- Publish main wrapper last

## Files to copy into another repo

- `npm/main-wrapper/`
- `npm/platform-packages/`
- `scripts/stage_platform_packages.sh`

