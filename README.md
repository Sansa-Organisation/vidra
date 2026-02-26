<div align="center">

<br />

<img src="https://img.shields.io/badge/vidra-v0.1.6--alpha-blueviolet?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBvbHlnb24gcG9pbnRzPSIyMyA3IDEyIDIgMSA3IDEgMTcgMTIgMjIgMjMgMTciPjwvcG9seWdvbj48bGluZSB4MT0iMTIiIHkxPSIyMiIgeDI9IjEyIiB5Mj0iMTIiPjwvbGluZT48L3N2Zz4=" alt="Vidra" />
<img src="https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust" alt="Rust" />
<img src="https://img.shields.io/badge/gpu-wgpu-green?style=for-the-badge" alt="GPU" />
<img src="https://img.shields.io/badge/wasm-browser-yellow?style=for-the-badge&logo=webassembly" alt="WASM" />
<img src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge" alt="License" />

<br />
<br />

# 🎬 Vidra

### Programmable Video Infrastructure

**Define video in code. Render with GPU acceleration. Deploy anywhere.**

*One engine. Every interface. Any scale.*

<br />

[Getting Started](#-getting-started) · [Features](#-features) · [Visual Editor](#-visual-editor) · [Web Scenes](#-web-scenes) · [VidraScript](#-vidrascript) · [TypeScript SDK](#-typescript-sdk) · [Browser Player](#-browser-player) · [MCP & AI](#-mcp-server--ai-integration) · [Architecture](#-architecture) · [Docs](#-documentation) · [Contributing](#-contributing)

<br />

</div>

---

## ✨ What is Vidra?

Vidra is a **programmable, AI-native video infrastructure platform**. Instead of dragging timelines, you define video in code — then render it with GPU acceleration, collaborate in real-time, and deploy to any surface: CLI, web, cloud, or edge.

```bash
vidra init my-project && cd my-project && vidra render main.vidra
```

**Vidra is to video what React is to UI** — a declarative, composable, deterministic system that turns video production into software engineering.

### Why Vidra?

| Traditional Video Tools | Vidra |
|-------------------------|-------|
| Manual timeline editing | Declarative code → deterministic output |
| Proprietary formats | Open IR specification (JSON) |
| Desktop-only rendering | GPU, WASM, cloud, edge — everywhere |
| AI bolted-on | AI primitives are first-class (`tts()`, `autocaption()`) |
| No version control | Text files — git-native, diffable, reviewable |
| Siloed workflows | One IR, every surface: CLI, SDK, MCP, visual editor |

---

## 🚀 Getting Started

### Install

```bash
# One-line install
curl -fsSL https://vidra.dev/install.sh | sh

# Via npm (restricted, org-scoped)
npx @sansavision/vidra@latest --help

# Or build from source
cargo install --path crates/vidra-cli
```

### Your First Video

```bash
vidra init hello-world && cd hello-world
vidra render main.vidra        # → output.mp4
vidra dev main.vidra            # → live preview with hot-reload
vidra editor main.vidra --open  # → visual editor in browser
```

### Your First VidraScript

```javascript
project(1920, 1080, 60) {
    scene("intro", 3s) {
        layer("background") {
            solid(#1a1a2e)
        }

        layer("title") {
            text("Hello, Vidra!", font: "Inter", size: 72, color: #e94560)
            position(960, 540)
            animation(opacity, from: 0, to: 1, duration: 1s, easing: ease-out)
            animation(scale_x, from: 0.8, to: 1.0, duration: 1.2s, easing: spring)
        }
    }
}
```

> 📖 **Full walkthrough:** [docs/quickstart.md](docs/quickstart.md)

---

## 🎯 Features

<table>
<tr>
<td width="50%">

### 🎨 VidraScript Language
A purpose-built DSL for declarative video composition — scenes, layers, animations, components, variants, responsive layouts, and brand variables.

→ [Language Reference](docs/vidrascript.md)

</td>
<td width="50%">

### ⚡ GPU-Accelerated Rendering
Real-time rendering powered by `wgpu` — works across NVIDIA, AMD, Apple Silicon, and Intel. Deterministic, byte-for-byte reproducible output.

→ [Architecture Guide](docs/architecture.md)

</td>
</tr>
<tr>
<td>

### 🌐 Web Scenes
Embed **any** web content as a compositable video layer — React components, D3 charts, Three.js scenes, plain HTML. Frame-accurate capture via headless CDP.

→ [Web Scenes Guide](docs/web-scenes.md)

</td>
<td>

### 🖥️ Visual Editor
`vidra editor` launches a full local visual editing environment — canvas preview, timeline scrubber, scene graph, property inspector, and integrated code editor.

→ [Editor Quickstart](docs/quickstart.md#6-visual-editor)

</td>
</tr>
<tr>
<td>

### 🤖 AI-Native Pipeline
First-class `tts()` and `autocaption()` primitives baked into the render graph. Text-to-speech via OpenAI/ElevenLabs and auto-captioning via Whisper — as native layer types.

→ [AI Workflows](docs/ai_workflows.md)

</td>
<td>

### 🔌 MCP Server (15 tools)
Full [Model Context Protocol](https://modelcontextprotocol.io) integration — any AI agent can create projects, add scenes, edit layers, generate web code, apply brands, and render.

→ [MCP & AI Docs](docs/ai_workflows.md)

</td>
</tr>
<tr>
<td>

### 🧩 Component System
Reusable, parameterized video components with props, variants, and slots. Build once, compose everywhere. Reference brand kits with `@brand.*`.

→ [Language Reference](docs/vidrascript.md)

</td>
<td>

### 📐 Responsive Video
Layout rules with `when aspect(...)` for automatic multi-format output — one source file, every platform (16:9, 9:16, 1:1, 4:5).

→ [Language Reference](docs/vidrascript.md)

</td>
</tr>
<tr>
<td>

### 🌍 Browser Player (WASM)
Render Vidra videos at 60fps in the browser — no server required. Write VidraScript or use the SDK, compile via WASM, play on `<canvas>`.

→ [Player Docs](#-browser-player)

</td>
<td>

### 🟦 TypeScript SDK
Build videos programmatically with the fluent `Project → Scene → Layer` builder API. Outputs VidraScript, IR JSON, or rendered MP4.

→ [SDK Reference](#-typescript-sdk)

</td>
</tr>
<tr>
<td>

### 🤝 Real-Time Collaboration
CRDT-based multiplayer editing at the IR level. Presence indicators, cursor sharing, conflict-free merging across clients.

→ [IR Specification](docs/ir-spec.md)

</td>
<td>

### 🔧 Plugin System & VidraFX
Extend the engine with WASM plugins for custom layer types, effects, and easings. Write GPU shaders in VidraFX DSL (`.vfx` files).

→ [Architecture Guide](docs/architecture.md)

</td>
</tr>
</table>

---

## 🖥️ Visual Editor

Launch a full visual editing environment with a single command:

```bash
vidra editor my-project.vidra --open
```

The editor provides:

| Panel | Description |
|-------|-------------|
| **Canvas** | Server-rendered GPU preview — frames streamed as JPEG over WebSocket, pan and zoom enabled |
| **Timeline** | Scrub through frames, play/pause, visual frame counter |
| **Scene Graph** | Expandable tree of all scenes and layers with selection |
| **Properties** | Live inspector for the selected layer — edit position, scale, opacity, content |
| **AI Chat** | Built-in AI assistant to generate scenes, modify layers, or plan projects |
| **MCP Console** | Manually trigger Model Context Protocol remote function calls against the project |
| **Code Editor** | Inline `.vidra` source editing with live compilation via Monaco |
| **Toolbar** | Undo/redo, layout toggle, export to MP4 |

The editor is a React app embedded directly into the Rust CLI binary — no separate Node.js runtime required to use it. Changes edit the `.vidra` source file on disk. The file watcher recompiles and pushes updated frames to all connected clients in real-time.

> 📖 **Full guide:** [docs/quickstart.md](docs/quickstart.md)

---

## 🌐 Web Scenes

Embed any web content inside a video — React dashboards, D3 charts, Three.js scenes, plain HTML/CSS:

```javascript
project(1920, 1080, 30) {
    scene("dashboard", 10s) {
        layer("background") { solid(#0f0f23) }

        layer("chart") {
            web(source: "./web/chart.html", viewport: 1920x1080, mode: "capture")
        }
    }
}
```

### Two Rendering Modes

| Mode | Behavior | Best For |
|------|----------|----------|
| **`capture`** | Frame-accurate — overrides `requestAnimationFrame`, `Date.now()`, `performance.now()` for deterministic output | Exported videos, CI renders |
| **`realtime`** | Screenshots at native fps intervals | Live preview, interactive content |

### The `window.__vidra` Bridge

Web content automatically receives a bridge object with timeline sync:

```javascript
const { frame, time, fps, vars } = window.__vidra;
// Use vars for data binding, time for animation sync
```

### Integration Packages

| Package | Purpose |
|---------|---------|
| `@sansavision/vidra-web-capture` | React hook `useVidraScene()` + vanilla `VidraCapture` class |

> 📖 **Full guide:** [docs/web-scenes.md](docs/web-scenes.md)
>
> 📂 **Examples:** [examples/web_chart.vidra](examples/web_chart.vidra) · [examples/web_react.vidra](examples/web_react.vidra) · [examples/web_interactive.vidra](examples/web_interactive.vidra)

---

## 📝 VidraScript

VidraScript is Vidra's domain-specific language for video composition:

| Feature | Syntax | Description |
|---------|--------|-------------|
| **Scenes** | `scene("name", 5s) { ... }` | Time-bounded segments of the timeline |
| **Layers** | `layer("title") { text("Hello") }` | Renderable units, stacked bottom-to-top |
| **Assets** | `asset Image("bg", "bg.png")` | Content-addressed media references |
| **Animations** | `animation(opacity, from: 0, to: 1, duration: 1s)` | Keyframe-driven property animations |
| **Components** | `component(Card, title: String) { ... }` | Reusable, parameterized building blocks |
| **Variants** | `variant("dark") { ... }` | Alternate component presentations |
| **Responsive** | `layout rules { when aspect(9:16) { ... } }` | Multi-format output from one source |
| **Brand Refs** | `color: @brand.primary` | Centralized design token references |
| **AI Nodes** | `tts("Hello", "en-US")` / `autocaption(@narration)` | Native AI-powered layer types |
| **Web Layers** | `web(source: "./dist", viewport: 1920x1080)` | Embedded browser content |
| **Conditionals** | `if (show_cta) { layer("cta") { ... } }` | Dynamic composition logic |
| **Effects** | `effect(blur, 5.0)` / `effect(removeBackground)` | Built-in and AI-powered effects |

> 📖 **Full reference:** [docs/vidrascript.md](docs/vidrascript.md)

---

## 🟦 TypeScript SDK

Build videos programmatically using the fluent TypeScript API (`@sansavision/vidra-sdk`):

```typescript
import { Project, Scene, Layer, Easing } from "@sansavision/vidra-sdk";

const project = new Project(1920, 1080, 60);
const scene = new Scene("intro", 3.0);

scene.addLayers(
    new Layer("bg").solid("#1a1a2e"),
    new Layer("title")
        .text("Hello!", "Inter", 100, "#ffffff")
        .position(960, 540)
        .animate("opacity", 0, 1, 1.0, Easing.EaseOut),
    new Layer("live_chart")
        .web("./web/chart.html", { viewport: [1920, 1080], mode: "capture" }),
);

project.addScene(scene);
project.toVidraScript();   // → VidraScript DSL string
project.toJSON();          // → IR JSON object
```

---

## 🌍 Browser Player

Render Vidra videos at 60fps in the browser using the WASM player (`@sansavision/vidra-player`):

```bash
cd packages/vidra-player && npm install && npm run demo
# → http://localhost:3456/examples/demo.html
```

```typescript
import { VidraEngine, Project, Scene, Layer } from "@sansavision/vidra-player";

const engine = new VidraEngine(canvas);
await engine.init();

engine.loadSource('project(1920, 1080, 60) { ... }');
engine.play();
```

The demo supports both **VidraScript** and **SDK** input modes. Web layers are rendered in sandboxed `<iframe>` elements and composited onto the main canvas.

---

## 🤖 MCP Server & AI Integration

Vidra includes a built-in [Model Context Protocol](https://modelcontextprotocol.io) server, enabling any AI agent to programmatically create and edit video:

```bash
vidra mcp   # Start the MCP server over stdio
```

### Available Tools (15)

| Tool | Description |
|------|-------------|
| `vidra.create_project` | Scaffold a new project with config |
| `vidra.add_scene` | Add a scene to the timeline |
| `vidra.edit_layer` | Edit layer properties by semantic path |
| `vidra.set_style` | Update styling on any target |
| `vidra.apply_brand_kit` | Apply a brand kit globally |
| `vidra.add_asset` | Register a media asset |
| `vidra.render_preview` | Trigger a local preview render |
| `vidra.storyboard` | Generate a storyboard from a text prompt |
| `vidra.list_templates` | Browse available starter templates |
| `vidra.add_resource` | Pull from Vidra Commons |
| `vidra.list_resources` | Search the resource library |
| `vidra.share` | Create a shareable link |
| `vidra.generate_web_code` | Save HTML/React code to `web/` for embedding |
| `vidra.add_web_scene` | Add a web layer to a scene |
| `vidra.edit_web_scene` | Edit viewport, source, mode, or variables on a web layer |

### Claude Desktop / Cursor Setup

```json
{
  "mcpServers": {
    "vidra": {
      "command": "npx",
      "args": ["@sansavision/vidra@latest", "mcp"]
    }
  }
}
```

### AI Primitives in VidraScript

```javascript
layer("narration") { tts("Welcome to the future", voice: "en-US-Journey-F") }
layer("captions")  { autocaption("assets/voiceover.mp3", font: "Inter", size: 48, color: #ffffff) }
layer("cutout")    { image("person.png")  effect(removeBackground) }
```

> 📖 **Full guide:** [docs/ai_workflows.md](docs/ai_workflows.md) · [LLM System Prompt](docs/llm.md) · [Quick LLM Prompt](docs/small-llm.md)

---

## 🏗 Architecture

```
crates/
├── vidra-core       # Core types, color, transforms, duration
├── vidra-lang       # VidraScript lexer, parser, checker, compiler, formatter
├── vidra-ir         # Intermediate Representation — the universal scene graph
├── vidra-render     # GPU rendering pipeline (wgpu), effects, compositing
├── vidra-encode     # FFmpeg encoding (H.264, H.265, ProRes, VP9, AV1, GIF, APNG, WebM)
├── vidra-web        # Web capture engine (Playwright/CDP headless browser)
├── vidra-fx         # VidraFX DSL → WGSL shader compiler
├── vidra-lsp        # Language Server Protocol for editor integration
├── vidra-wasm       # WebAssembly module — browser rendering
└── vidra-cli        # CLI, MCP server, editor backend, auth, cloud

packages/
├── vidra-sdk           # @sansavision/vidra-sdk — TypeScript builder API
├── vidra-player        # @sansavision/vidra-player — WASM browser player
├── vidra-web-capture   # @sansavision/vidra-web-capture — React hook for web scenes
├── vidra-editor        # Visual editor frontend (React + Zustand + Vite)
└── npm-release         # Platform release packaging
```

### The Vidra Pipeline

```
VidraScript / TypeScript SDK / MCP / Visual Editor
                    ↓
          [ vidra-lang: Lexer → Parser → Checker → Compiler ]
                    ↓
              [ Vidra IR (JSON) ]
             ↙                ↘
    [ vidra-render ]      [ vidra-wasm ]
    [ GPU + wgpu ]        [ <canvas> 60fps ]
         ↓                      ↓
    [ vidra-encode ]       [ Browser Player ]
    [ .mp4 / .mov ]
```

Every input surface compiles to the same **Vidra IR** — a queryable, composable, deterministic scene graph.

> 📖 **Deep dives:** [Architecture](docs/architecture.md) · [IR Specification](docs/ir-spec.md) · [Web Scenes Architecture](docs/web-scenes.md)

---

## 🖥 CLI Reference

```
USAGE: vidra <COMMAND>

COMMANDS:
  render <file>             Render a .vidra file to video
  render <file> --cloud     Submit to cloud render cluster
  check <file>              Parse and type-check (no render)
  dev <file>                Start live preview server with hot-reload
  editor <file> [--open]    Launch the visual editor in your browser
  init <name>               Scaffold a new project
  init <name> --kit <k>     Scaffold with a starter kit
  fmt <file>                Auto-format VidraScript
  inspect <file>            Print the compiled IR tree as JSON
  test <file>               Run snapshot tests
  bench <file>              Benchmark render performance
  add <template>            Install from Vidra Commons
  storyboard <prompt>       Generate an AI storyboard
  share [file]              Create a shareable link
  publish <path>            Publish to Vidra Commons
  mcp                       Start the MCP server (stdio)
  auth login                Authenticate with Vidra Cloud
  workspace create <name>   Create a team workspace
  plugins list              List installed plugins
  dashboard                 View render metrics
  doctor                    Environment health check
  info                      Version and engine info
```

---

## 📊 Performance

| Metric | Target | Actual |
|--------|--------|--------|
| 1080p60 solid color render | < 50ms/frame | ✅ ~12ms/frame |
| 4K text + shapes | < 100ms/frame | ✅ ~45ms/frame |
| Cold start to first frame | < 2s | ✅ ~1.2s |
| IR compilation | < 100ms | ✅ ~15ms |
| Conformance suite | 10/10 pass | ✅ 10/10 |

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [**Quickstart Guide**](docs/quickstart.md) | Install, create, render, preview, and launch the editor |
| [**VidraScript Reference**](docs/vidrascript.md) | Full language specification — content types, animations, components, web layers |
| [**Architecture Guide**](docs/architecture.md) | System design, crate breakdown, GPU pipeline, editor architecture |
| [**IR Specification**](docs/ir-spec.md) | Open Intermediate Representation — types, validation, CRDT protocol |
| [**Web Scenes Guide**](docs/web-scenes.md) | Embedding web content — capture modes, bridge API, React/D3/Three.js integration |
| [**AI & MCP Workflows**](docs/ai_workflows.md) | MCP server setup, AI primitives (TTS, captions), local providers, Claude integration |
| [**LLM System Prompt**](docs/llm.md) | Comprehensive context document for AI code generation |
| [**LLM Quick Prompt**](docs/small-llm.md) | Concise system prompt for smaller/faster models |

---

## 🧪 Testing

```bash
# Full workspace test suite
cargo test

# Local quality gate (requires no CI)
npm run local:ci

# By crate
cargo test -p vidra-lang       # Language tests
cargo test -p vidra-render     # GPU conformance suite
cargo test -p vidra-ir         # IR serialization tests
cargo test -p vidra-web        # Web capture tests

# Editor frontend
cd packages/vidra-editor && npm run build

# Benchmarks
cargo bench -p vidra-render
```

---

## 📦 npm Packages

| Package | Registry | Description |
|---------|----------|-------------|
| `@sansavision/vidra` | ![npm](https://img.shields.io/badge/npm-restricted-red) | CLI wrapper (native binary per-platform) |
| `@sansavision/vidra-sdk` | ![npm](https://img.shields.io/badge/npm-restricted-red) | TypeScript SDK — fluent builder API |
| `@sansavision/vidra-player` | ![npm](https://img.shields.io/badge/npm-restricted-red) | WASM browser player + engine |
| `@sansavision/vidra-web-capture` | ![npm](https://img.shields.io/badge/npm-restricted-red) | React hook for web scenes (`useVidraScene`) |

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

## 🤝 Contributing

We welcome contributions! Whether it's a bug fix, new feature, component, or documentation improvement:

1. Fork the repo
2. Create a branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

Please read our [Architecture Guide](docs/architecture.md) and [IR Specification](docs/ir-spec.md) if you're working on the engine internals.

---

<div align="center">

<br />

**Built with ❤️ and Rust**

*One engine. Every interface. Any scale.*

<br />

</div>
