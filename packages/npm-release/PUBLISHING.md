# Publishing checklist (template)

## 0) Versions

All packages MUST share the same version `X.Y.Z`:
- main wrapper: `npm/main-wrapper/package.json`
- every platform package: `npm/platform-packages/*/package.json`

## 1) Build native binaries

Build your binaries and create archives in `dist/`.

## 2) Stage binaries into platform packages

Edit variables in `scripts/stage_platform_packages.sh`, then run:

```bash
./scripts/stage_platform_packages.sh
```

## 3) Publish platform packages first

```bash
cd npm/platform-packages/darwin-arm64 && npm publish
cd npm/platform-packages/darwin-x64 && npm publish
cd npm/platform-packages/linux-x64-musl && npm publish
cd npm/platform-packages/linux-arm64-musl && npm publish
cd npm/platform-packages/win32-x64-msvc && npm publish
```

## 4) Publish main wrapper last

```bash
cd npm/main-wrapper && npm publish
```

If you publish the wrapper first, installs may fail to resolve the matching optionalDependency.
