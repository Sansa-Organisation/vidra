# Platform packages

Create one folder per platform package. Each folder contains:

- `package.json` with `name`, `version`, `os`, `cpu`
- `bin/` containing the native executables

This template mirrors Atlas’s targets:
- darwin-arm64
- darwin-x64
- linux-x64-musl
- linux-arm64-musl
- win32-x64-msvc
