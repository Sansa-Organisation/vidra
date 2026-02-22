# @sansavision/vidra-examples

A showcase application demonstrating the power of the Vidra WASM player. This repository provides a beautiful, git-cloneable example of integrating `@sansavision/vidra-player` into a modern Vite web application using TypeScript.

## Features

- **VidraScript Editor**: Write and instantly compile custom DSL code.
- **JavaScript SDK Builder**: Programmatically create scenes, layers, and animations with the fluent API.
- **Asset Manager**: Drag-and-drop support for images and media, which uses `VidraEngine.loadImageAsset` to effortlessly inject external assets straight into the WASM cache.
- **Interactive Player**: Frame accurate seeking, playback controls, and project metadata all within an impressive glass-morphism aesthetic UI.

## Project Structure

- `index.html`: The layout and styling markup.
- `src/main.ts`: The core logic that bridges the UI and the `VidraEngine`.
- `src/style.css`: A premium glassmorphism dark-mode aesthetic for impressing investors.
- `vite.config.ts`: Contains `vite-plugin-wasm` and `vite-plugin-top-level-await` plugins required for natively loading the Vidra Rust/WASM binary in the browser.

## Getting Started

To grab just this example project without cloning the entire repository, you can cleanly extract it using `degit`:

```bash
npx degit sansavision/vidra/packages/vidra-examples my-vidra-project
cd my-vidra-project
npm install
npm run dev
```

If you already have the repository cloned, simply navigate to this directory:

```bash
cd packages/vidra-examples
npm install
npm run dev
```

## Deployment

The project can be built statically using:
```bash
npm run build
\`\`\`
The resulting \`dist/\` folder can be uploaded to any static hosting provider.
