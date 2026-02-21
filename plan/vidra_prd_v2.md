# Vidra — Product Requirements Document (PRD)

**Version:** 2.0
**Status:** Internal Draft
**Last Updated:** 2026-02-20
**Authors:** Vidra Founding Team

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Vision & Mission](#3-vision--mission)
4. [Goals & Success Metrics](#4-goals--success-metrics)
5. [Target Users](#5-target-users)
6. [Core Value Proposition](#6-core-value-proposition)
7. [Strategic Architecture — Local-First, Cloud-Later](#7-strategic-architecture--local-first-cloud-later)
8. [Product Pillars](#8-product-pillars)
9. [The Vidra Engine — Technical Foundation](#9-the-vidra-engine--technical-foundation)
10. [The Vidra IR — Video as a Data Structure](#10-the-vidra-ir--video-as-a-data-structure)
11. [VidraScript — A Typed DSL for Video](#11-vidrascript--a-typed-dsl-for-video)
12. [Feature Roadmap — MVP through V3](#12-feature-roadmap--mvp-through-v3)
13. [S-Tier Developer Features](#13-s-tier-developer-features)
14. [Non-Developer Experience — LLM-Native Creation via MCP](#14-non-developer-experience--llm-native-creation-via-mcp)
15. [Game Developer Pipeline](#15-game-developer-pipeline)
16. [Performance Guarantees & SLAs](#16-performance-guarantees--slas)
17. [Composition Model — Video as Components](#17-composition-model--video-as-components)
18. [Multi-Target Responsive Output](#18-multi-target-responsive-output)
19. [Marketplace & Ecosystem](#19-marketplace--ecosystem)
20. [The Unified UX/DX Story](#20-the-unified-uxdx-story)
21. [CLI & Developer Workflow](#21-cli--developer-workflow)
22. [Competitive Landscape](#22-competitive-landscape)
23. [Monetization Strategy](#23-monetization-strategy)
24. [Risks & Mitigations](#24-risks--mitigations)
25. [Phased Roadmap & Milestones](#25-phased-roadmap--milestones)
26. [Positioning & Brand](#26-positioning--brand)
27. [Appendix](#27-appendix)

---

## 1. Executive Summary

**Vidra** is a programmable, AI-native video infrastructure platform that enables developers, creators, game studios, and businesses to generate, edit, and render video through code, visual tools, or natural language.

Vidra is not a video editor. **Vidra is video infrastructure.**

The platform is built around a Rust-powered, GPU-accelerated rendering engine with a local-first architecture. Developers interact through a typed DSL (VidraScript), a CLI, and an SDK. Non-technical users interact through LLM-powered natural language interfaces via the Model Context Protocol (MCP). Game developers interact through parameterized asset pipelines with native engine export.

All interfaces compile to the same internal representation — the **Vidra IR** — a queryable, composable, deterministic scene graph that serves as the universal language for video.

**One engine. Every interface. Any scale.**

---

## 2. Problem Statement

### The Current State of Video

Video creation today is slow, manual, impossible to scale, non-programmable, and extremely difficult to automate. The entire industry is built on timeline-based editors designed for a single human operator dragging clips on a screen. This model breaks completely for the workloads that define the next era of video.

### Workloads That Existing Tools Cannot Serve

- **AI-generated video** — LLMs and generative models need a programmatic way to compose, arrange, and render video. No timeline editor exposes an API that an AI agent can drive.
- **Personalization at scale** — Generating 10,000 variants of a product video with different names, prices, and localized text is a manual nightmare in every existing tool.
- **Programmatic marketing** — Dynamic ad creative that responds to real-time data (inventory, pricing, user segments) requires video that is generated on-the-fly, not pre-rendered.
- **Dynamic content pipelines** — Media companies, e-commerce platforms, and social networks need video generated as part of automated data pipelines, not human-operated editing sessions.
- **Game asset production** — Game developers need high volumes of 2D animated assets (UI animations, effects, cutscenes, trailers) and the pipeline for producing these is fragmented and slow.

### Why Existing Tools Fail

| Problem | Impact |
|---|---|
| Timeline-based paradigm | Cannot be automated or called via API |
| Browser-based rendering (Remotion) | Fundamentally limited in performance, no real GPU acceleration |
| Cloud-only platforms | Expensive, latency-bound, vendor lock-in risk |
| No composability | Every video is a monolith; nothing is reusable |
| No testing or CI/CD integration | Video is the only software artifact with zero automated quality assurance |
| No AI-native interface | LLMs cannot drive existing tools without brittle hacks |

---

## 3. Vision & Mission

### Vision

Create the world's first:

> **Composable, local-first, AI-native video runtime.**

A system where video is treated as a compiled software artifact — defined in code, tested in CI, rendered deterministically, and deployable to any target.

### Mission

Make video as programmable, testable, and composable as software — accessible to developers through code, to non-developers through conversation, and to machines through protocol.

---

## 4. Goals & Success Metrics

### Primary Goals

1. Make video programmable like software — defined in code, version-controlled, tested, and deployed through standard engineering workflows.
2. Deliver best-in-class rendering performance — 10-50x faster than browser-based alternatives on equivalent hardware.
3. Enable non-technical users to create professional video through natural language via LLM + MCP.
4. Become the default rendering engine for AI video tools, content platforms, and game asset pipelines.
5. Build a local-first platform that works without internet, with an optional cloud layer for collaboration and managed rendering.

### Success Metrics

| Metric | Target (V1) | Target (V2) |
|---|---|---|
| Time to first render (new user) | < 60 seconds | < 30 seconds |
| Local render speed (60s 1080p video) | < 5 seconds | < 2 seconds |
| Preview hot-reload latency | < 500ms | < 200ms |
| Render determinism | Bit-identical across platforms | Bit-identical across platforms |
| SDK weekly active developers | 5,000 | 50,000 |
| MCP monthly active non-dev users | — | 20,000 |
| Marketplace published components | — | 10,000 |
| Game studio adopters | 50 | 500 |
| Render jobs per day (platform-wide) | 100,000 | 10,000,000 |

---

## 5. Target Users

### Phase 1 — Developer Wedge

**Primary:** Developers building AI video tools, automated video systems, content platforms, and creative SaaS. These users choose tools based on performance, DX, and API quality. They are the foundation of the ecosystem.

**Secondary:** Game developers producing 2D animated assets, UI animations, promotional trailers, and in-game video textures. They need parameterized, batch-exportable, engine-native output.

### Phase 2 — AI-Powered Expansion

**Non-technical creators** who interact with Vidra through LLM-powered natural language interfaces (Claude, GPT, local models) via MCP. They never see code, never see a timeline, but produce professional-quality video through conversation.

**Small businesses and solo creators** who use brand kits and conversational storyboarding to produce marketing content without hiring an agency.

### Phase 3 — Enterprise & Teams

**Agencies, media companies, and marketing teams** who use collaborative features, template systems, and managed cloud rendering at scale.

**Design and brand teams** who curate component libraries and enforce brand consistency across all video output.

---

## 6. Core Value Proposition

| Old Model | Vidra Model |
|---|---|
| Timeline editing | Declarative video spec (VidraScript) |
| Manual export | CLI / API render |
| Local proprietary software | Open local engine + optional cloud |
| Static, monolithic video | Composable, parameterized components |
| No testing | Visual regression testing in CI |
| No AI integration | LLM-native creation via MCP |
| Browser-based rendering | Rust + GPU-accelerated engine |
| One format at a time | Multi-target responsive output |
| Opaque render process | Full render observability and profiling |

---

## 7. Strategic Architecture — Local-First, Cloud-Later

### Why Local-First

Vidra adopts a local-first architecture for V1. This is a deliberate strategic choice, not a constraint.

**Cost discipline.** GPU cloud infrastructure is brutally expensive pre-revenue. Local-first eliminates infrastructure cost while the team focuses on making the core engine exceptional.

**Developer trust.** Developers are skeptical of cloud-only platforms from startups. Local-first means their pipelines work even if Vidra the company disappears. This removes the single largest adoption barrier for infrastructure tools.

**Performance validation.** The rendering engine, IR, and SDK are the hard technical bets. Local-first forces these to be genuinely excellent rather than hiding performance problems behind cloud scale.

**Feedback velocity.** Users rendering locally hit bugs faster, iterate faster, and provide feedback faster. No queues, no cold starts, no infra incidents.

### The Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: INFRASTRUCTURE (cloud, usage-based)           │
│  Managed rendering, auto-scaling, CDN delivery          │
│  "vidra render --cloud"                                 │
│  Target: Phase 2+                                       │
├─────────────────────────────────────────────────────────┤
│  Layer 2: PLATFORM (cloud, freemium)                    │
│  Collaboration, asset management, marketplace,          │
│  team workspaces, render analytics, version history     │
│  Target: Phase 1.5+                                     │
├─────────────────────────────────────────────────────────┤
│  Layer 1: ENGINE (local, source-available)              │
│  Rust renderer, GPU acceleration, VidraScript,          │
│  CLI tooling, SDK, IR compiler                          │
│  "vidra dev" / "vidra render"                           │
│  Target: Phase 1 (V1 launch)                            │
└─────────────────────────────────────────────────────────┘
```

**Layer 1 — The Engine** runs entirely on the user's machine. This is the Rust renderer, the IR compiler, the SDK, the CLI, and the preview server. It is the product that developers fall in love with. It has zero cloud dependencies.

**Layer 2 — The Platform** is the optional cloud layer for collaboration, asset management, template marketplace, team workspaces, render analytics, and version history. Users opt into this because it makes their workflow better, not because the local experience is crippled.

**Layer 3 — The Infrastructure** is managed cloud rendering for users who lack GPUs, need scale, or need renders triggered via API in production. This is the high-margin business at maturity. Migration from local to cloud is a single config change: `render: local` → `render: cloud`.

### Critical Design Constraint

The IR and render graph must be **location-agnostic** from day one. A render job must not know or care whether it executes on a laptop or a 50-node cluster. Only the execution layer changes. This prevents architectural forks and ensures the transition from local to cloud is seamless.

### Deterministic Rendering

Every render must be **content-addressable**. Same input IR produces the same output bytes, every time, on any supported hardware. This is enforced through a conformance test suite — a set of reference renders that must produce bit-identical output across all supported platforms (NVIDIA, AMD, Apple Silicon). Determinism unlocks aggressive caching, version diffing, and verifiable builds.

---

## 8. Product Pillars

### Pillar 1: Programmable Video Engine

A rendering engine that compiles video definitions into optimized execution graphs, runs on the user's local GPU, and produces deterministic output.

### Pillar 2: Multi-Interface Creation

Users can create video through code (VidraScript / SDK), visual tools (web editor, component playground), and natural language (LLM + MCP). All interfaces compile to the same IR. No interface is second-class.

### Pillar 3: Composition-First Design

Video is built from composable, reusable, parameterized components — not monolithic timelines. Components nest, override, version independently, and are shareable through the marketplace.

### Pillar 4: Performance as a Feature

Rendering performance is not aspirational — it is specified, measured, profiled, and regression-tested. The platform ships with built-in benchmarking, profiling, and observability tooling.

### Pillar 5: AI-Native Architecture

The system is designed to be controlled by AI agents natively through MCP. The IR is both the developer interface and the AI interface. There is no separate "AI mode."

---

## 9. The Vidra Engine — Technical Foundation

### Core Engine

| Component | Technology | Rationale |
|---|---|---|
| Renderer | Rust | Memory safety, performance, no GC pauses |
| Frame computation | SIMD intrinsics | Vectorized pixel operations |
| GPU acceleration | wgpu (WebGPU API) | Cross-platform GPU access (Vulkan, Metal, DX12) |
| Shader system | WGSL + custom DSL | Portable GPU shaders |
| Audio engine | Rust-native mixer | Sample-accurate sync with video |
| Encoding | FFmpeg bindings + native AV1 | Broad format support with modern codec optimization |

### Execution Architecture

```
User Input (VidraScript / SDK / MCP / Visual Editor)
        │
        ▼
    ┌─────────┐
    │  Parser  │  ── Validates and parses input into AST
    └────┬────┘
         ▼
    ┌──────────┐
    │ Compiler │  ── Compiles AST to Vidra IR (scene graph)
    └────┬─────┘
         ▼
    ┌────────────┐
    │ Optimizer  │  ── Dead-layer elimination, constant folding,
    └────┬───────┘     frame deduplication, cache resolution
         ▼
    ┌──────────────┐
    │ Render Graph │  ── DAG of GPU/CPU operations per frame
    └────┬─────────┘
         ▼
    ┌────────────┐
    │  Executor  │  ── Local GPU dispatch OR cloud job scheduler
    └────┬───────┘
         ▼
    ┌───────────┐
    │  Encoder  │  ── H.264, H.265, VP9, AV1, ProRes, image sequences
    └────┬──────┘
         ▼
    ┌──────────┐
    │  Output  │  ── File, stream, sprite sheet, HLS/DASH
    └──────────┘
```

### Performance Targets

| Operation | Target | Notes |
|---|---|---|
| Cold start to preview | < 2 seconds | Engine initialization + first frame |
| Hot-reload preview update | < 500ms (V1), < 200ms (V2) | IR diff → re-render only changed frames |
| Full render (60s, 1080p) | < 5 seconds | On modern discrete GPU |
| Full render (60s, 4K) | < 15 seconds | On modern discrete GPU |
| Memory ceiling (1080p) | < 2 GB VRAM | Streaming asset decode |
| Render throughput (batch) | 200+ jobs/hour | Single workstation |

### Smart Diff Engine

The preview hot-reload system is powered by a diff engine at the IR level. When a user changes a single property, the engine computes the minimal set of frames that are affected, re-renders only those frames, and composites them into the existing preview buffer. This is the architectural foundation of sub-second preview and it is what makes the development experience feel instant.

---

## 10. The Vidra IR — Video as a Data Structure

The Vidra IR (Intermediate Representation) is the most important technical decision in the platform. It is the equivalent of the DOM for the web — the canonical, queryable, composable data structure that represents a video.

### Design Principles

1. **Semantic addressing** — Every element has a stable, human-readable path: `video.scenes[2].layers["logo"].opacity`. AI agents, scripts, and APIs can surgically modify any part of a video without understanding the whole structure.

2. **Composable** — Scenes, layers, effects, and components nest arbitrarily. The IR is a tree, not a flat list.

3. **Deterministic** — The same IR always produces the same output. The IR is content-hashable.

4. **Serializable** — The IR serializes to a human-readable format (JSON/YAML) and a compact binary format for performance.

5. **Queryable** — The IR supports path-based queries, filter expressions, and structural pattern matching. You can ask "give me all text layers in scene 3 with font size > 24" and get a result set.

6. **Extensible** — Plugins and custom components register their own node types in the IR schema without forking the core.

### IR Node Types (Core)

```
Project
├── Settings (resolution, fps, duration, color space)
├── Assets (registry of fonts, images, audio, video clips)
├── BrandKit (colors, fonts, logos, motion style)
├── Scene[]
│   ├── Duration
│   ├── Transition (in/out)
│   └── Layer[]
│       ├── Type (text, image, video, shape, component, AI-node)
│       ├── Transform (position, scale, rotation, anchor)
│       ├── Style (fill, stroke, opacity, blend mode)
│       ├── Animation[]
│       │   ├── Property (target path)
│       │   ├── Keyframes[]
│       │   └── Easing
│       ├── Effects[]
│       │   ├── Type (blur, glow, chromatic aberration, etc.)
│       │   └── Parameters
│       └── Children[] (nested layers / components)
└── Audio[]
    ├── Source
    ├── Timeline placement
    └── Effects (fade, EQ, ducking)
```

---

## 11. VidraScript — A Typed DSL for Video

VidraScript is a purpose-built, statically-typed domain-specific language for describing video. It is not a general-purpose language — it is a language optimized for expressing video compositions, animations, and render configurations.

### Why a DSL

An SDK in TypeScript or Python gives you video-as-library. A DSL gives you video-as-language. The difference is tooling: VidraScript gets autocomplete, type checking, linting, static analysis, and error messages that reference video concepts, not programming concepts. It lowers the barrier for creative technologists who aren't full-stack developers.

### Design Goals

- Concise and readable for common operations (a lower-third should be 5 lines, not 50)
- Statically typed with inference (catch errors before rendering)
- Compiles to Vidra IR (it's syntactic sugar, not a runtime)
- Interoperable with TypeScript/Python SDKs (use VidraScript for composition, host language for logic)

### Example

```vidra
import { FadeIn, SlideUp } from "vidra/transitions"
import { LowerThird } from "@vidra-marketplace/broadcast-kit"

project(1920x1080, 30fps) {

  scene("intro", 5s) {
    layer("background") {
      image("./assets/hero.jpg")
      animation(scale, from: 1.0, to: 1.05, ease: easeInOut)
    }

    layer("title") {
      text("Introducing Vidra", font: "Inter Bold", size: 72)
      position(center, center)
      enter(FadeIn, delay: 1s, duration: 0.8s)
    }

    layer("subtitle") {
      text("Video, compiled.", font: "Inter Light", size: 32, color: #AAAAAA)
      position(center, center + 60)
      enter(SlideUp, delay: 1.8s, duration: 0.6s)
    }
  }

  scene("features", 8s) {
    layer("lower-third") {
      LowerThird(
        name: "Jane Smith",
        title: "CEO, Vidra",
        style: "minimal-dark"
      )
    }
  }
}
```

### Tooling

- **`vidra check`** — static analysis, type checking, and linting
- **LSP server** — autocomplete, hover docs, go-to-definition in VS Code, Neovim, etc.
- **`vidra fmt`** — opinionated auto-formatter
- **Playground** — browser-based VidraScript editor with live preview

---

## 12. Feature Roadmap — MVP through V3

### MVP (Phase 0 — Prototype)

Core engine proof-of-concept. Internal use only.

- Rust rendering engine (single-threaded, single GPU)
- VidraScript parser and compiler to IR
- Basic IR → render graph → frame output pipeline
- CLI: `vidra render <file>` produces an MP4
- 5 built-in layer types: text, image, video, shape, color
- Basic keyframe animation system
- H.264 encoding via FFmpeg
- Reference conformance test suite (10 test cases)

### V1 (Phase 1 — Developer Release)

The product developers download, try, and adopt. Local-first.

- **Engine:** Multi-threaded rendering, GPU acceleration via wgpu, SIMD frame computation
- **VidraScript:** Full type system, LSP server, formatter, linter
- **SDK:** TypeScript and Python SDKs compiling to IR
- **CLI:** `vidra init`, `vidra dev`, `vidra render`, `vidra check`, `vidra bench`, `vidra test`, `vidra inspect`
- **Preview server:** Local dev server with sub-500ms hot-reload
- **Component system:** Parameterized, composable video components with props
- **Template system:** Pre-built templates as installable packages
- **Asset pipeline:** Upload and reference images, video clips, fonts, audio
- **Testing:** Visual regression snapshot testing (`vidra test`)
- **Profiling:** Render benchmarking and performance profiling (`vidra bench`)
- **Inspect:** Visual debugger with render tree, frame-level GPU inspection (`vidra inspect`)
- **Time-travel debugging:** Replayable render traces with frame-level step-through
- **Export formats:** H.264, H.265, ProRes, VP9, AV1, PNG sequence, GIF
- **Import interop:** FFmpeg filter graphs, Lottie/Rive, image sequences
- **Deterministic rendering:** Content-addressable output with conformance suite (100+ test cases)
- **Multi-target output:** Single source renders to 16:9, 9:16, 1:1, 4:5 via layout rules
- **Video Storybook:** Component playground with adjustable props and live preview
- **Documentation:** Comprehensive guides, API reference, tutorials, and example projects
- **Game dev support:** Sprite sheet export, texture sequences, parameterized batch variants

### V2 (Phase 2 — Platform + AI)

Cloud layer, LLM-native creation, collaboration, marketplace.

- **MCP server:** First-party Model Context Protocol server exposing all Vidra capabilities as LLM tools
- **Conversational storyboarding:** Text-to-storyboard via MCP before rendering
- **Brand Kit:** Persistent brand context (colors, fonts, logos, motion style) injected into MCP
- **`vidra share`:** One-link preview with timestamped commenting and feedback loop
- **Cloud rendering:** Managed render infrastructure, usage-based pricing, `vidra render --cloud`
- **Collaboration:** CRDT-based multiplayer editing across code and visual interfaces
- **Marketplace:** Community-published components, templates, effects, and audio
- **AI copilot editor:** Visual editor with inline AI assistance
- **Semantic editing:** Natural language commands applied to the IR ("make the intro faster", "change the color scheme to warm tones")
- **Asset intelligence:** Auto-tagging, smart cropping, content-aware layout
- **Render streaming:** Progressive output — video begins playing before render completes
- **Version history:** Full project history with visual diffs between versions
- **Render observability dashboard:** Traces, metrics, GPU profiling per render job
- **Plugin system:** Third-party extensions registered in the IR schema
- **Native AI pipeline hooks:** AI models (style transfer, object detection, generative fill) as first-class render graph nodes with shared GPU memory
- **GitHub integration:** Renders triggered on PR, visual diffs in review, deploy-on-merge

### V3 (Phase 3 — Ecosystem + Edge)

Open ecosystem, edge runtime, enterprise.

- **Edge runtime:** WASM-compiled lightweight renderer for edge nodes (Cloudflare Workers, Fastly Compute) — personalized video rendered at the CDN edge in < 100ms
- **Public IR spec:** Open specification for the Vidra IR, enabling third-party tools to read/write Vidra projects
- **Community runtime ports:** Community-maintained renderer implementations for specialized hardware
- **Third-party plugins:** Open plugin API with sandboxed execution
- **Enterprise features:** SSO, audit logs, role-based access, SLA guarantees
- **Live collaboration protocol:** Open CRDT protocol for real-time multi-user editing from any client
- **Broadcast integration:** Live video output to RTMP/SRT for streaming platforms
- **After Effects import:** .aep project file parsing and conversion to Vidra IR

---

## 13. S-Tier Developer Features

These features define Vidra's identity as a developer tool and create defensible differentiation.

### 13.1 Sub-Second Preview (Hot Reload for Video)

When a developer changes a line of VidraScript or adjusts a parameter, the preview updates in under 500ms. Powered by the IR diff engine — only re-render frames that actually changed. This is Vidra's "Vite moment." Every other tool forces you to wait. Vidra doesn't.

### 13.2 `vidra inspect` — X-Ray Vision for Video

A CLI and visual debugger that lets developers see inside any render. Hover over any frame and see the full render tree: which layers are composited, what shaders are running, where time is spent, what the GPU is doing. Click any visual element and see its full lineage back to the VidraScript that produced it. No video tool on earth has this.

### 13.3 Time-Travel Debugging

Every render produces a replayable trace. When something looks wrong at frame 847, you don't re-render — you scrub to that frame in the debugger and step through the render graph execution. Inspect intermediate buffer states, shader outputs, and composition results at any point in the pipeline.

### 13.4 `vidra test` — Visual Regression Testing

Built-in snapshot testing for video. Define key frames or time ranges as test assertions. On every code change, Vidra renders those frames and diffs them pixel-by-pixel against the baseline, with configurable tolerance thresholds. If your brand intro drifts by one pixel after a refactor, CI catches it.

### 13.5 Video Storybook — Component Playground

A local dev server that renders every video component in isolation with adjustable props. Your lower-third component shows up with sliders for duration, color, text length. Your transition component shows up with dropdowns for easing curves. Designers and developers share this as a living catalog of everything the system can produce.

### 13.6 `vidra bench` — Performance Profiling

One command benchmarks your project across resolutions, durations, and hardware profiles. Structured report showing render time per scene, GPU memory peaks, asset decode bottlenecks, and frame-over-frame cost distribution. Flags regressions against your last run. Commit the baseline to git; CI tells you when a PR makes renders slower.

### 13.7 Render Observability — Traces, Metrics, Profiling

Every render job emits structured traces: which frames were slow, which assets took longest to decode, where GPU utilization dropped, memory high-water marks. Exposed through a dashboard and API. This is how you earn trust from engineering teams running Vidra in production.

### 13.8 Render Streaming — Progressive Output

Don't make users wait for a full render. Stream encoded frames as they're produced using chunked encoding and out-of-order frame assembly. A 60-second video starts playing back within seconds of the render starting.

### 13.9 Live Collaboration Protocol

A CRDT-based protocol for real-time multiplayer editing at the IR level. One person in VS Code, another in the visual editor — both see each other's changes in real time. The protocol is open so third-party editors can plug in.

### 13.10 Escape Hatch Interop Layer

One-command import from FFmpeg filter graphs, Remotion projects, Lottie/Rive animations, Apple Motion templates. One-command export to ProRes, H.264/5, VP9, AV1, GIF, image sequences, MPEG-DASH/HLS. The exit door is what gets people to walk in.

---

## 14. Non-Developer Experience — LLM-Native Creation via MCP

### The Core Idea

Anyone with access to an LLM can create, edit, and render video through natural conversation. The LLM writes VidraScript so the human doesn't have to. The IR is both the developer interface and the AI interface. There is no separate "AI mode."

### 14.1 Vidra MCP Server

A first-party Model Context Protocol server exposing every Vidra capability as a tool. Available tools include:

| Tool | Description |
|---|---|
| `vidra.create_project` | Initialize a new video project with settings |
| `vidra.add_scene` | Add a scene with layers, timing, and transitions |
| `vidra.edit_layer` | Modify any layer property via semantic path |
| `vidra.set_style` | Apply visual style (colors, typography, mood) |
| `vidra.apply_brand_kit` | Apply a saved brand kit to the project |
| `vidra.storyboard` | Generate a visual storyboard from text description |
| `vidra.render_preview` | Render a low-res preview for review |
| `vidra.render_final` | Render full-quality output |
| `vidra.add_asset` | Import an image, video, audio, or font |
| `vidra.list_templates` | Browse available templates and components |
| `vidra.share` | Generate a shareable preview link |

### Example Workflow

**User (in Claude):** *"Make me a 30-second product launch video for my new sneaker brand. The vibe is dark, minimal, with quick cuts."*

**Claude** calls `vidra.storyboard` → generates a visual storyboard with 6 key frames. User reviews, says *"Love frames 1-4, frame 5 needs more energy, cut frame 6."* Claude calls `vidra.create_project`, `vidra.add_scene` (×5), `vidra.set_style`, `vidra.render_preview`. User sees the preview, says *"Make the text bigger and add a bass-heavy sound effect on the transitions."* Claude calls `vidra.edit_layer` and `vidra.add_asset` to make surgical edits.

The user never sees code. The user never sees a timeline. The output is professional.

### 14.2 Conversational Storyboarding

Before rendering, the MCP server generates a lightweight storyboard — a grid of static key frames with timing annotations — from a text description. Users iterate on the concept before a single frame is rendered. This dramatically reduces the cost of exploration and makes the LLM interaction feel collaborative.

### 14.3 Brand Kit as Context

Users define a brand kit (colors, fonts, logos, motion style, audio signatures) that is injected into the MCP server's context. Every video the LLM creates automatically adheres to brand guidelines without the user specifying them each time. Set it up once; every video is on-brand by default.

### 14.4 `vidra share` — One-Link Preview and Feedback

After rendering, the user gets a shareable link with an embedded player and a timestamped comment layer. Reviewers leave feedback directly on the video. The LLM consumes that feedback through MCP and makes edits. The entire review cycle — create, share, get feedback, revise — happens without anyone touching an editor.

---

## 15. Game Developer Pipeline

### The Problem

Game developers need high volumes of 2D animated assets: UI animations, cutscene segments, promotional trailers, in-game video textures, loading screen animations, ability/spell effects. The current pipeline involves After Effects → manual export → manual format conversion → import to engine → realize it looks wrong → repeat.

### 15.1 Sprite Sheet and Texture Sequence Export

Render any Vidra composition to a packed sprite sheet or numbered image sequence optimized for game engines. Direct export formats for Unity (sprite atlas), Unreal (flipbook texture), and Godot (AnimatedSprite2D). A game dev writes VidraScript for a fire effect and exports it directly into their engine's asset pipeline.

### 15.2 Parameterized Asset Variants

Define an animation once with parameters (color, speed, scale, intensity) and batch-render hundreds of variants in one command. A game dev who needs a damage number popup in 12 colors, 4 sizes, and 3 speeds gets 144 sprite sheets from a single definition.

```vidra
component DamagePopup(color: Color, size: px, speed: Duration) {
  layer("number") {
    text("999", font: "Impact", size: size, color: color)
    animation(position.y, from: 0, to: -80, duration: speed, ease: easeOut)
    animation(opacity, from: 1.0, to: 0.0, delay: speed * 0.6, duration: speed * 0.4)
  }
}

batch render DamagePopup {
  color: [#FF4444, #44FF44, #4444FF, #FFAA00, #FF44FF, #44FFFF,
          #FFFFFF, #FFD700, #FF6600, #00FF88, #FF0088, #8844FF]
  size: [24px, 32px, 48px, 64px]
  speed: [0.4s, 0.6s, 0.8s]
  export: spritesheet(format: "unity", padding: 2px)
}
```

### 15.3 Procedural Animation Nodes

Built-in GPU-accelerated nodes for common game asset patterns: particle systems, procedural noise (Perlin, simplex, Worley), sine-wave distortions, glow and bloom, chromatic aberration, screen shake, glitch effects, dissolve transitions, outline/silhouette rendering. These compose natively with all other layers and effects.

### 15.4 Engine-Aware Preview

A preview mode that simulates how the asset will appear inside a target game engine — with the correct color space, compression artifacts, mip levels, and target frame rate. What you see in Vidra preview matches what you get in-engine.

---

## 16. Performance Guarantees & SLAs

Performance is not a feature bullet point. It is a contract.

### Local Rendering SLAs (V1, on recommended hardware)

| Metric | Guarantee |
|---|---|
| Engine cold start | < 2 seconds |
| Preview first frame | < 1 second after cold start |
| Hot-reload latency | < 500ms for single-property change |
| 1080p 30fps render | > 120 fps render speed (4x real-time) |
| 4K 30fps render | > 30 fps render speed (1x real-time) |
| Peak VRAM (1080p) | < 2 GB |
| Peak VRAM (4K) | < 6 GB |
| Peak RAM | < 4 GB |

### Benchmarking Protocol

Every release is benchmarked against the Vidra Conformance Suite (VCS) — a set of standardized video compositions at varying complexity levels. Benchmark results are published publicly with every release. Performance regressions block releases.

### Supported Hardware

**V1 Launch:**
- NVIDIA GPUs: RTX 2060 and above
- AMD GPUs: RX 6600 and above
- Apple Silicon: M1 and above (via Metal backend)

**V2 Target:**
- Intel Arc GPUs
- Integrated GPUs (reduced feature set, lower resolution caps)
- CPU-only fallback (for CI/CD environments without GPU)

---

## 17. Composition Model — Video as Components

### The Concept

Vidra treats video the way React treats UI: composable, reusable components with typed props. A lower-third is a component. A transition is a component. A branded intro is a component with `{title}`, `{date}`, `{logo}` as inputs.

### Component Definition

```vidra
component BrandedIntro(
  title: String,
  subtitle: String = "",
  date: String = today(),
  logo: Image = @brand.logo,
  duration: Duration = 4s,
  color_primary: Color = @brand.primary
) {
  scene("branded-intro", duration) {
    layer("bg") {
      solid(color_primary)
    }
    layer("logo") {
      image(logo, width: 120)
      position(center, center - 40)
      enter(FadeIn, duration: 0.6s)
    }
    layer("title") {
      text(title, font: @brand.heading_font, size: 56, color: #FFFFFF)
      position(center, center + 40)
      enter(SlideUp, delay: 0.4s, duration: 0.5s)
    }
    if subtitle != "" {
      layer("subtitle") {
        text(subtitle, font: @brand.body_font, size: 28, color: #CCCCCC)
        position(center, center + 80)
        enter(FadeIn, delay: 0.8s, duration: 0.4s)
      }
    }
    layer("date") {
      text(date, font: @brand.body_font, size: 18, color: #888888)
      position(right - 40, bottom - 30)
      enter(FadeIn, delay: 1.2s)
    }
  }
}
```

### Component Features

- **Typed props** with defaults and validation
- **Conditional rendering** (if/else in composition)
- **Nesting** — components can contain other components
- **Slots** — components can accept child content
- **Variants** — `component.variant("dark")` applies a predefined prop override set
- **Versioning** — components are independently versioned and upgradeable
- **Publishing** — components are publishable to the Vidra Marketplace

---

## 18. Multi-Target Responsive Output

### The Problem

Every marketing team needs the same video in vertical (9:16), horizontal (16:9), square (1:1), and story (4:5) formats. Today, this means four separate editing passes. Vidra solves this at the IR level.

### Layout Rules

VidraScript supports layout rules that define how a composition adapts to different aspect ratios — similar to responsive CSS.

```vidra
layout rules {
  when aspect(16:9) {
    layer("title") { position(left + 80, center) }
    layer("logo") { position(right - 60, top + 40) }
  }
  when aspect(9:16) {
    layer("title") { position(center, top + 200); size(90%w, auto) }
    layer("logo") { position(center, bottom - 100); scale(0.8) }
  }
  when aspect(1:1) {
    layer("title") { position(center, center - 40) }
    layer("logo") { position(center, top + 40); scale(0.7) }
  }
}
```

### Render Command

```bash
vidra render --targets 16:9,9:16,1:1,4:5 --output ./exports/
```

One command, four outputs. Each structurally adapted, not just cropped.

---

## 19. Marketplace & Ecosystem

### What Gets Published

- **Components** — reusable video building blocks (lower-thirds, intros, transitions, effects)
- **Templates** — full video compositions with parameterized placeholders
- **Procedural nodes** — custom effects, generators, and filters
- **Brand kits** — curated style packages (color palettes, font pairings, motion presets)
- **Audio packs** — music, sound effects, and ambient tracks licensed for use
- **Plugins** — IR extensions and custom render nodes

### Revenue Model

Marketplace operates on a revenue share model. Free components drive ecosystem growth; premium components drive creator revenue and platform margin.

### Quality Standards

All marketplace submissions pass automated review: renders without errors, meets conformance tests, documentation is present, props are typed and documented.

---

## 20. The Unified UX/DX Story

### For Developers

Vidra is the Rust-powered, GPU-accelerated video engine with the best CLI tooling in any creative domain. You write VidraScript or use the TypeScript/Python SDK. You get instant previews with hot-reload. You have real debugging tools — inspect, time-travel, profiling. You ship with confidence via visual regression tests in CI. It's local-first, you own your pipeline, and it's faster than anything else because it's built on a real rendering engine.

### For Non-Developers

Vidra is the thing your AI assistant uses to make videos for you. You describe what you want in natural language, review storyboards and previews, give feedback in conversation, and professional-quality video comes out. You never see code. You never see a timeline. You never feel limited. Your brand kit ensures everything is consistent.

### For Game Developers

Vidra is the programmable motion graphics pipeline that speaks your language — parameters, nodes, batch export, engine-native formats. It replaces the After Effects → export → import → realize-it's-wrong loop with a code-defined, CI-integrated asset pipeline that produces exactly what your engine needs.

### For Teams

Vidra is the collaborative platform where developers build the system, designers curate the component library, marketers generate content through conversation, and everything stays on-brand and version-controlled. The visual editor and the code editor are two views of the same IR.

---

## 21. CLI & Developer Workflow

### Commands

| Command | Description |
|---|---|
| `vidra init <name>` | Scaffold a new project with sensible defaults |
| `vidra dev` | Start local preview server with hot-reload |
| `vidra render` | Render locally, full quality |
| `vidra render --cloud` | Send to managed cloud render (V2+) |
| `vidra render --targets` | Multi-target responsive render |
| `vidra check` | Static analysis, type checking, linting |
| `vidra fmt` | Auto-format VidraScript files |
| `vidra test` | Run visual regression snapshot tests |
| `vidra test --update` | Update snapshot baselines |
| `vidra bench` | Performance benchmark with regression detection |
| `vidra inspect` | Open visual render debugger |
| `vidra inspect --frame 847` | Jump to specific frame in debugger |
| `vidra share` | Generate shareable preview link (V2+) |
| `vidra publish` | Publish component to marketplace (V2+) |
| `vidra add <package>` | Install component from marketplace |
| `vidra export --spritesheet` | Export as packed sprite sheet |
| `vidra export --sequence` | Export as numbered image sequence |

### Zero-to-First-Render

```bash
# Install
curl -fsSL https://vidra.dev/install.sh | sh

# Create project
vidra init my-first-video
cd my-first-video

# Start dev server — preview opens in browser
vidra dev

# Edit main.vidra in your editor of choice
# Preview updates in < 500ms on save

# Render final output
vidra render
# → output/my-first-video.mp4 (rendered in seconds)
```

**Target: under 60 seconds from install to seeing your first rendered frame.**

### GitHub-Native Workflow (V2)

- Renders triggered on pull request
- Visual diffs embedded in PR review (frame-by-frame comparison)
- Deploy-on-merge to CDN for hosting
- Video becomes a CI/CD artifact like any other build output

---

## 22. Competitive Landscape

| | Vidra | Remotion | FFmpeg | After Effects | Runway |
|---|---|---|---|---|---|
| **Rendering** | Rust + GPU | Headless Chrome | CLI pipeline | Local app | Cloud |
| **Performance** | 10-50x browser | Baseline | Fast but no composition | Slow render | Queue-based |
| **Programmability** | Full (DSL + SDK + API) | SDK only | CLI flags | ExtendScript (limited) | Prompt only |
| **Local-first** | Yes | Yes | Yes | Yes | No |
| **AI-native (MCP)** | Yes | No | No | No | Partial |
| **Composability** | Component model | React components | None | Precomps (limited) | None |
| **Testing/CI** | Built-in | Manual | Manual | None | None |
| **Debugging** | Inspect + time-travel | Browser DevTools | None | Limited | None |
| **Game dev export** | Native | Manual | Manual | Manual | No |
| **Multi-target** | Layout rules | Manual | Manual | Manual | No |
| **Deterministic** | Guaranteed | No (browser variance) | Mostly | No | No |

### Positioning Against Remotion

Remotion proved the market for code-defined video. Vidra takes the same distribution strategy (local-first, developer-friendly, code-driven) and pairs it with a fundamentally superior engine. Remotion is limited by the browser rendering model — it's clever but slow, can't do real GPU-accelerated effects, and has no video-native IR. Vidra is to Remotion what Node.js was to PHP: same developer-friendly ethos, completely different runtime that unlocks things the old approach cannot do.

---

## 23. Monetization Strategy

### Phase 1 — Free and Open

The engine (Layer 1) is free and source-available. This drives adoption. Revenue: $0. Cost: minimal (no infrastructure).

### Phase 2 — Platform Freemium

The platform (Layer 2) launches with a freemium model.

| Tier | Price | Includes |
|---|---|---|
| Free | $0 | Local engine, 5 shared previews/month, community components |
| Pro | $29/month | Unlimited sharing, cloud preview, brand kits, version history |
| Team | $79/seat/month | Collaboration, shared libraries, team analytics, priority support |

### Phase 3 — Cloud Infrastructure (Usage-Based)

Managed rendering (Layer 3) is priced per render-second.

| Resolution | Price |
|---|---|
| 720p | $0.005 / render-second |
| 1080p | $0.01 / render-second |
| 4K | $0.04 / render-second |

Volume discounts and committed-use pricing for enterprise customers.

### Phase 3+ — Marketplace Revenue Share

Platform takes 20% of premium component and template sales. Creators retain 80%.

---

## 24. Risks & Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **Rendering engine complexity** | High | Incremental delivery: single-threaded first, GPU second, distributed third. Conformance test suite gates every release. |
| **GPU compatibility across hardware** | High | wgpu abstraction layer covers Vulkan/Metal/DX12. Conformance tests run on all target hardware. CPU fallback for CI environments. |
| **Deterministic rendering variance** | High | Content-addressable output verified by conformance suite. Bit-identical rendering is a hard requirement, not a goal. |
| **Encoding performance** | Medium | FFmpeg bindings for broad support; invest in native AV1 encoder for performance-critical paths. |
| **Market education** | Medium | Developer advocacy, public benchmarks, open-source components. Lead with "it's like Remotion but 10-50x faster" narrative. |
| **Local-first → no recurring revenue** | Medium | Platform layer (collaboration, sharing, brand kits) provides natural cloud value. Cloud rendering is an expansion, not a paywall. |
| **Remotion entrenchment** | Medium | Performance differentiation is measurable and dramatic. Import tool eases migration. Target use cases Remotion can't serve (GPU effects, game assets, AI pipeline hooks). |
| **LLM/MCP adoption uncertainty** | Low | MCP is additive to the core developer product. If LLM adoption is slower than expected, the developer product stands on its own. |
| **Becoming "just another Remotion"** | Medium | VidraScript DSL, inspect/debug tooling, game dev pipeline, and AI-native architecture create clear category separation from day one. |

---

## 25. Phased Roadmap & Milestones

### Phase 0 — Prototype (Months 1-3)

**Goal:** Prove the engine works.

- [ ] Rust render pipeline: parse → IR → render graph → frames → MP4
- [ ] VidraScript parser (subset)
- [ ] 5 core layer types (text, image, video, shape, solid)
- [ ] Basic keyframe animation
- [ ] CLI: `vidra render`
- [ ] 10-case conformance test suite
- [ ] Internal demo: render a 30-second branded intro from VidraScript

**Milestone:** A VidraScript file renders to an MP4 that is correct and fast.

### Phase 1 — Developer Release (Months 4-9)

**Goal:** Developers adopt Vidra for real projects.

- [ ] GPU acceleration (wgpu)
- [ ] Multi-threaded rendering
- [ ] Full VidraScript type system + LSP server
- [ ] TypeScript SDK
- [ ] Preview server with hot-reload (< 500ms)
- [ ] `vidra init`, `vidra dev`, `vidra check`, `vidra fmt`
- [ ] `vidra test` — visual regression testing
- [ ] `vidra bench` — performance profiling
- [ ] `vidra inspect` — visual debugger
- [ ] Component system with typed props
- [ ] Multi-target responsive output
- [ ] Sprite sheet and texture sequence export
- [ ] Parameterized batch rendering
- [ ] Import: Lottie, image sequences, FFmpeg filters
- [ ] Export: H.264, H.265, ProRes, VP9, AV1, PNG sequence, GIF
- [ ] 100+ case conformance test suite
- [ ] Public documentation site
- [ ] Public benchmark results

**Milestone:** 1,000 developers using Vidra weekly. Public benchmark showing 10x+ performance over Remotion.

### Phase 1.5 — Platform Soft Launch (Months 8-12)

**Goal:** Introduce cloud value, validate monetization.

- [ ] `vidra share` — shareable preview links with comments
- [ ] Brand kit system
- [ ] Cloud preview rendering (low-res)
- [ ] Team workspaces
- [ ] Version history
- [ ] Pro tier launch ($29/month)
- [ ] Component marketplace (curated, first-party)

**Milestone:** 500 paying Pro users. $15K MRR.

### Phase 2 — AI & Cloud (Months 10-18)

**Goal:** Non-developers can create video. Cloud rendering at scale.

- [ ] Vidra MCP server (first-party)
- [ ] Conversational storyboarding
- [ ] Managed cloud rendering (usage-based)
- [ ] CRDT-based collaboration protocol
- [ ] Community marketplace
- [ ] AI copilot in visual editor
- [ ] Native AI pipeline hooks (models as render graph nodes)
- [ ] Render streaming (progressive output)
- [ ] GitHub Actions integration
- [ ] Python SDK
- [ ] Plugin system (third-party IR extensions)
- [ ] Team tier launch ($79/seat/month)

**Milestone:** 10,000 weekly active developers. 5,000 MCP-driven renders per day. $100K MRR.

### Phase 3 — Ecosystem & Edge (Months 18-30)

**Goal:** Vidra is default video infrastructure. Open ecosystem.

- [ ] Edge runtime (WASM-compiled renderer for CDN edge)
- [ ] Public IR specification
- [ ] Open collaboration protocol
- [ ] Enterprise tier (SSO, audit, SLA)
- [ ] After Effects .aep import
- [ ] Broadcast output (RTMP/SRT)
- [ ] Community runtime ports
- [ ] Third-party plugin sandbox

**Milestone:** 50,000 weekly active developers. 1M+ daily render jobs. $1M MRR.

---

## 26. Positioning & Brand

### Positioning Statement

Vidra is not a video editor. Vidra is not an AI video generator.

**Vidra is video infrastructure** — the programmable, local-first, AI-native runtime that makes video as composable, testable, and deployable as software.

### Tagline

> **"One engine. Every interface. Any scale."**

### Alternative Taglines

- "Video, compiled."
- "The runtime for video."
- "Build video like software."
- "The API for motion."

### Narrative Arc

1. **For developers:** "You wouldn't build a web app by manually dragging divs around a screen. Why do you build video that way? Vidra makes video programmable."
2. **For non-developers:** "Describe the video you want. Your AI assistant builds it. You review and refine through conversation. That's it."
3. **For the industry:** "The web got its infrastructure layer decades ago. Video never did. Vidra is that layer."

---

## 27. Appendix

### A. Glossary

| Term | Definition |
|---|---|
| **Vidra IR** | Intermediate Representation — the canonical, queryable scene graph that represents a video |
| **VidraScript** | Vidra's typed domain-specific language for defining video compositions |
| **Render Graph** | The DAG of GPU/CPU operations compiled from the IR for a given frame range |
| **MCP** | Model Context Protocol — the standard for exposing tools to LLMs |
| **Component** | A reusable, parameterized video building block with typed props |
| **Brand Kit** | A persistent set of brand assets and style rules (colors, fonts, logos, motion) |
| **Conformance Suite** | A set of reference renders used to verify deterministic output across hardware |
| **Hot-Reload** | Sub-second preview updates when source code changes |
| **Sprite Sheet** | A packed grid of animation frames used in game engines |

### B. Recommended Hardware (V1)

| Component | Minimum | Recommended |
|---|---|---|
| GPU | NVIDIA RTX 2060 / AMD RX 6600 / Apple M1 | NVIDIA RTX 4070 / Apple M3 Pro |
| VRAM | 4 GB | 8 GB+ |
| RAM | 8 GB | 16 GB+ |
| Storage | SSD (any) | NVMe SSD |
| OS | Linux, macOS, Windows 10+ | Linux, macOS (best DX) |

### C. Supported Export Formats

| Format | Type | Use Case |
|---|---|---|
| H.264 (.mp4) | Video | Universal playback |
| H.265/HEVC (.mp4) | Video | High efficiency, HDR |
| ProRes (.mov) | Video | Professional post-production |
| VP9 (.webm) | Video | Web delivery |
| AV1 (.mp4) | Video | Next-gen web delivery |
| PNG Sequence | Image | Compositing, game engines |
| Sprite Sheet (.png) | Image | Game engines (Unity, Unreal, Godot) |
| GIF | Image | Social, messaging |
| MPEG-DASH / HLS | Streaming | Adaptive bitrate delivery |
| RTMP / SRT | Live | Broadcast, live streaming (V3) |

---

*This document is a living artifact. It will evolve as we build, learn, and ship. The principles — local-first, performance-first, composable, AI-native — do not change. Everything else is subject to iteration.*

**— The Vidra Team**
