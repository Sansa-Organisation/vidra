<div align="center">

<br />

<img src="https://img.shields.io/badge/vidra-v0.1.0-blueviolet?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBvbHlnb24gcG9pbnRzPSIyMyA3IDEyIDIgMSA3IDEgMTcgMTIgMjIgMjMgMTciPjwvcG9seWdvbj48bGluZSB4MT0iMTIiIHkxPSIyMiIgeDI9IjEyIiB5Mj0iMTIiPjwvbGluZT48L3N2Zz4=" alt="Vidra" />
<img src="https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust" alt="Rust" />
<img src="https://img.shields.io/badge/gpu-wgpu-green?style=for-the-badge" alt="GPU" />
<img src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge" alt="License" />

<br />
<br />

# 🎬 Vidra

### Programmable Video Infrastructure

**Define video in code. Render with GPU acceleration. Deploy anywhere.**

*One engine. Every interface. Any scale.*

<br />

[Getting Started](#-getting-started) · [Features](#-features) · [Architecture](#-architecture) · [VidraScript](#-vidrascript) · [TypeScript SDK](#-typescript-sdk) · [Browser Player](#-browser-player) · [MCP Server](#-mcp-server) · [Contributing](#-contributing)

<br />

</div>

---

## ✨ What is Vidra?

Vidra is a **programmable, AI-native video infrastructure platform**. Instead of dragging timelines, you define video in code — then render it with GPU acceleration, collaborate in real-time, and deploy to any surface: CLI, web, cloud, or edge.

```
vidra init my-project && cd my-project && vidra render main.vidra
```

**Vidra is to video what React is to UI** — a declarative, composable, deterministic system that turns video production into software engineering.

---

## 🚀 Getting Started

### Install

```bash
# One-line install
curl -fsSL https://vidra.dev/install.sh | sh

# Or build from source
cargo install --path crates/vidra-cli
```

### Your First Video

```bash
# Create a new project
vidra init hello-world
cd hello-world

# Render it
vidra render main.vidra

# Start live preview
vidra dev main.vidra
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
        }
    }
}
```

---

## 🎯 Features

### 🎨 VidraScript Language
A purpose-built DSL for declarative video composition with scenes, layers, animations, components, variants, and responsive layouts.

### ⚡ GPU-Accelerated Rendering
Real-time rendering powered by `wgpu` — works across NVIDIA, AMD, and Apple Silicon. Deterministic output guaranteed.

### 🧩 Component System
Reusable, parameterized video components with props, variants, slots, and semantic versioning. Build once, use everywhere.

### 🤖 AI-Native Pipeline
First-class `tts()` and `autocaption()` nodes in the render graph. Text-to-speech and auto-captioning as native layer types.

### 🔌 MCP Server
Full Model Context Protocol integration — any AI agent can create projects, add scenes, edit layers, apply brands, and trigger renders.

### 🎭 Brand Kit System
Define colors, fonts, logos, and motion styles once. Reference them with `@brand.*` across all projects.

### 📐 Responsive Video
Layout rules with `when aspect(...)` for automatic multi-format output — one source, every platform.

### ☁️ Cloud Rendering
`vidra render --cloud` submits to a managed GPU cluster. Usage-based pricing, CDN delivery, full job management.

### 🤝 Real-Time Collaboration
CRDT-based multiplayer editing at the IR level. Presence indicators, cursor sharing, conflict-free merging.

### 🔧 Plugin System
Extend the engine with WASM-sandboxed plugins. Custom layer types, effects, and animation easings.

### 📦 Vidra Commons
A community resource library — components, templates, fonts, sounds. Install with `vidra add <package>`.

### 🏢 Enterprise Ready
SSO (SAML/OIDC), audit logs, RBAC, team workspaces, machine seat licensing.

---

## 🏗 Architecture

Vidra is built as a modular Rust workspace:

```
crates/
├── vidra-core      # Core types, color, transforms, duration
├── vidra-lang      # VidraScript lexer, parser, checker, compiler, formatter
├── vidra-ir        # Intermediate Representation — the universal scene graph
├── vidra-render    # GPU rendering pipeline (wgpu), effects, compositing
├── vidra-encode    # FFmpeg-based encoding (H.264, H.265, ProRes, VP9, AV1)
├── vidra-lsp       # Language Server Protocol for editor integration
├── vidra-wasm      # WebAssembly module — browser rendering (CPU)
└── vidra-cli       # CLI application, MCP server, auth, receipts

packages/
├── vidra-sdk       # @sansavision/vidra-sdk — TypeScript builder API
└── vidra-player    # @sansavision/vidra-player — WASM browser player
```

### The Vidra Pipeline

```
VidraScript / TypeScript SDK / MCP
         ↓
    [ Parser + Checker ] ───── or ───── [ SDK → IR JSON ]
         ↓                                    ↓
    [ Compiler → IR ] ◀──────────────────────┘
         ↓                    ↓
    [ GPU Render ]      [ WASM Render ]
         ↓                    ↓
    [ .mp4 / .mov ]     [ <canvas> 60fps ]
```

Every input surface compiles to the same **Vidra IR** — a queryable, composable, deterministic scene graph. The [IR specification](docs/ir-spec.md) is open and documented.

---

## 📝 VidraScript

VidraScript is Vidra's domain-specific language for video composition:

| Feature | Syntax |
|---------|--------|
| **Scenes** | `scene("name", 5s) { ... }` |
| **Layers** | `layer("title") { text("Hello") }` |
| **Assets** | `asset Image("bg", "bg.png")` |
| **Animations** | `animation(opacity, from: 0, to: 1, duration: 1s)` |
| **Components** | `component(Card, title: String) { ... }` |
| **Variants** | `variant("dark") { ... }` |
| **Responsive** | `layout rules { when aspect(9:16) { ... } }` |
| **Brand Refs** | `color: @brand.primary` |
| **AI Nodes** | `tts("Hello", "en-US")` / `autocaption(@narration)` |
| **Conditionals** | `if (show_cta) { layer("cta") { ... } }` |

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
);
project.addScene(scene);

// Output options
project.toVidraScript();      // → VidraScript DSL string
project.toJSON();             // → IR JSON object
await project.render("out.mp4"); // → render to file via CLI
```

---

## 🌐 Browser Player

Render Vidra videos at 60fps in the browser using the WASM player (`@sansavision/vidra-player`):

```bash
# Run the demo
cd packages/vidra-player
npm install && npm run demo
# → http://localhost:3456/examples/demo.html
```

The demo supports two modes:
- **VidraScript tab** — write `.vidra` DSL, compile via WASM
- **JavaScript SDK tab** — use the fluent `Project`/`Scene`/`Layer` API directly

```typescript
import { VidraEngine, Project, Scene, Layer } from "@sansavision/vidra-player";

const engine = new VidraEngine(canvas);
await engine.init();

// Mode 1: VidraScript
engine.loadSource('project(1920, 1080, 60) { ... }');

// Mode 2: SDK Project object
const project = new Project(1920, 1080, 60);
// ... build scenes ...
engine.loadProject(project);

engine.play();
```

---

## 🤖 MCP Server

Vidra includes a built-in [Model Context Protocol](https://modelcontextprotocol.io) server, enabling any AI agent to programmatically create and edit video:

```bash
vidra mcp   # Start the MCP server over stdio
```

### Available Tools

| Tool | Description |
|------|-------------|
| `vidra.create_project` | Create a new project |
| `vidra.add_scene` | Add a scene to the timeline |
| `vidra.edit_layer` | Edit layer properties by semantic path |
| `vidra.set_style` | Update styling on any target |
| `vidra.apply_brand_kit` | Apply a brand kit |
| `vidra.add_asset` | Register a media asset |
| `vidra.render_preview` | Trigger a local preview render |
| `vidra.storyboard` | Generate a storyboard from text |
| `vidra.share` | Create a shareable link |
| `vidra.list_templates` | Browse available templates |
| `vidra.add_resource` | Pull from Vidra Commons |
| `vidra.list_resources` | Search the resource library |

---

## 🖥 CLI Reference

```
vidra render <file>          Render a .vidra file to video
vidra render <file> --cloud  Submit to cloud render cluster
vidra check <file>           Parse and type-check
vidra dev <file>             Start live preview server
vidra init <name>            Scaffold a new project
vidra init <name> --kit <k>  Scaffold with a starter kit
vidra fmt <file>             Auto-format VidraScript
vidra inspect <file>         Print the compiled IR tree
vidra test <file>            Run snapshot tests
vidra bench <file>           Benchmark render performance
vidra add <template>         Install from marketplace
vidra storyboard <prompt>    Generate an AI storyboard
vidra share [file]           Create a shareable link
vidra publish <path>         Publish to Vidra Commons
vidra mcp                    Start the MCP server
vidra auth login             Authenticate with Vidra Cloud
vidra workspace create <n>   Create a team workspace
vidra plugins list           List installed plugins
vidra dashboard              View render metrics
vidra doctor                 Environment health check
vidra info                   Version and engine info
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

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run language tests only
cargo test -p vidra-lang

# Run GPU conformance suite
cargo test -p vidra-render

# Run benchmarks
cargo bench -p vidra-render
```

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

Please read our [IR specification](docs/ir-spec.md) if you're working on the engine internals.

---

<div align="center">

<br />

**Built with ❤️ and Rust**

*One engine. Every interface. Any scale.*

<br />

</div>
