# Vidra — Product Requirements Document (PRD)

**Version:** 2.1
**Status:** Internal Draft
**Last Updated:** 2026-02-21
**Authors:** Vidra Founding Team
**Changelog:** v2.1 adds Hybrid Sync Architecture, Vidra License Token (VLT), Vidra Commons Resource Library, Starter Kits, Render Receipts, and updated monetization model.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Vision & Mission](#3-vision--mission)
4. [Goals & Success Metrics](#4-goals--success-metrics)
5. [Target Users](#5-target-users)
6. [Core Value Proposition](#6-core-value-proposition)
7. [Strategic Architecture — Local-First, Cloud-Coordinated](#7-strategic-architecture--local-first-cloud-coordinated)
8. [Hybrid Sync Architecture](#8-hybrid-sync-architecture)
9. [Product Pillars](#9-product-pillars)
10. [The Vidra Engine — Technical Foundation](#10-the-vidra-engine--technical-foundation)
11. [The Vidra IR — Video as a Data Structure](#11-the-vidra-ir--video-as-a-data-structure)
12. [VidraScript — A Typed DSL for Video](#12-vidrascript--a-typed-dsl-for-video)
13. [Feature Roadmap — MVP through V3](#13-feature-roadmap--mvp-through-v3)
14. [S-Tier Developer Features](#14-s-tier-developer-features)
15. [Non-Developer Experience — LLM-Native Creation via MCP](#15-non-developer-experience--llm-native-creation-via-mcp)
16. [Game Developer Pipeline](#16-game-developer-pipeline)
17. [Performance Guarantees & SLAs](#17-performance-guarantees--slas)
18. [Composition Model — Video as Components](#18-composition-model--video-as-components)
19. [Multi-Target Responsive Output](#19-multi-target-responsive-output)
20. [Vidra License Token (VLT) & Telemetry](#20-vidra-license-token-vlt--telemetry)
21. [Vidra Commons — Resource Library & Ecosystem](#21-vidra-commons--resource-library--ecosystem)
22. [Marketplace & Ecosystem](#22-marketplace--ecosystem)
23. [The Unified UX/DX Story](#23-the-unified-uxdx-story)
24. [CLI & Developer Workflow](#24-cli--developer-workflow)
25. [Competitive Landscape](#25-competitive-landscape)
26. [Monetization Strategy](#26-monetization-strategy)
27. [Risks & Mitigations](#27-risks--mitigations)
28. [Phased Roadmap & Milestones](#28-phased-roadmap--milestones)
29. [Positioning & Brand](#29-positioning--brand)
30. [Appendix](#30-appendix)

---

## 1. Executive Summary

**Vidra** is a programmable, AI-native video infrastructure platform that enables developers, creators, game studios, and businesses to generate, edit, and render video through code, visual tools, or natural language.

Vidra is not a video editor. **Vidra is video infrastructure.**

The platform is built around a Rust-powered, GPU-accelerated rendering engine with a **local-first, cloud-coordinated** architecture. Rendering always happens on the user's machine by default — the cloud provides project coordination, asset sync, resource library, collaboration, and sharing. Cloud GPU rendering is available as an optional premium upgrade for users who need it.

Developers interact through a typed DSL (VidraScript), a CLI, and an SDK. Non-technical users interact through LLM-powered natural language interfaces via the Model Context Protocol (MCP). Game developers interact through parameterized asset pipelines with native engine export.

All interfaces compile to the same internal representation — the **Vidra IR** — a queryable, composable, deterministic scene graph that serves as the universal language for video.

Every user — including free-tier users — authenticates with a **Vidra License Token (VLT)**, a signed offline-capable credential that enables plan enforcement, telemetry, and seamless sync between local and cloud environments.

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
5. Build a local-first platform that works without internet, with a cloud coordination layer for sync, collaboration, and resource sharing.
6. Keep infrastructure costs radically low by defaulting all rendering to user hardware, reserving cloud GPU for premium opt-in.

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
| Vidra Commons published resources | — | 50,000 |
| Game studio adopters | 50 | 500 |
| Render jobs per day (platform-wide) | 100,000 | 10,000,000 |
| Free-tier users with active VLT | 10,000 | 100,000 |
| Cloud render conversion rate | — | 5-10% of active users |

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
| Cloud GPU bills from day one | Local rendering by default; cloud as premium opt-in |
| Local proprietary software | Open local engine + cloud coordination layer |
| Static, monolithic video | Composable, parameterized components |
| No testing | Visual regression testing in CI |
| No AI integration | LLM-native creation via MCP |
| Browser-based rendering | Rust + GPU-accelerated engine |
| One format at a time | Multi-target responsive output |
| Opaque render process | Full render observability and profiling |
| Empty canvas on first launch | Rich resource library and starter kits |

---

## 7. Strategic Architecture — Local-First, Cloud-Coordinated

### Why Local-First

Vidra adopts a local-first architecture. This is a deliberate strategic choice, not a constraint.

**Cost discipline.** GPU cloud infrastructure is brutally expensive pre-revenue. By defaulting all rendering to the user's machine, Vidra's cloud costs reduce to object storage + a lightweight API server. This is 10-100x cheaper to operate than cloud render platforms. Even as the platform scales, the vast majority of compute happens on user hardware.

**Developer trust.** Developers are skeptical of cloud-only platforms from startups. Local-first means their pipelines work even if Vidra the company disappears. This removes the single largest adoption barrier for infrastructure tools.

**Performance validation.** The rendering engine, IR, and SDK are the hard technical bets. Local-first forces these to be genuinely excellent rather than hiding performance problems behind cloud scale.

**Feedback velocity.** Users rendering locally hit bugs faster, iterate faster, and provide feedback faster. No queues, no cold starts, no infra incidents.

### The Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: MANAGED GPU (cloud, usage-based, opt-in)      │
│  Cloud rendering for users without GPUs or needing       │
│  scale. Queue-based. Premium only.                       │
│  "vidra render --cloud"                                  │
│  Target: Phase 2+ (limited beta earlier)                 │
├─────────────────────────────────────────────────────────┤
│  Layer 2: COORDINATION PLATFORM (cloud, free + paid)     │
│  Project sync, asset management, resource library,       │
│  collaboration, sharing, dashboards, render receipts,    │
│  team workspaces. All users get this.                    │
│  Target: Phase 1.5+                                      │
├─────────────────────────────────────────────────────────┤
│  Layer 1: ENGINE (local, source-available)                │
│  Rust renderer, GPU acceleration, VidraScript,           │
│  CLI tooling, SDK, IR compiler. Runs on user hardware.   │
│  "vidra dev" / "vidra render"                            │
│  Target: Phase 1 (V1 launch)                             │
└─────────────────────────────────────────────────────────┘
```

**Layer 1 — The Engine** runs entirely on the user's machine. This is the Rust renderer, the IR compiler, the SDK, the CLI, and the preview server. It has zero cloud dependencies for core rendering functionality.

**Layer 2 — The Coordination Platform** is the cloud layer available to all users (including free tier). It stores project metadata, asset manifests, render receipts, and shared resources. It enables sync, collaboration, sharing, and discovery. It never performs rendering — it coordinates. Users authenticate with a Vidra License Token (VLT) to access this layer.

**Layer 3 — Managed GPU** is cloud rendering for users who don't have local GPUs, need burst scale, or need headless API-triggered rendering. This is the premium tier. It runs the same engine that runs locally — the only difference is where the compute happens. Access is gated, initially offered to beta testers, and later to paying users via a queue system with usage-based pricing.

### The Cost Advantage

| Platform | Cloud cost model |
|---|---|
| Runway, Synthesia, etc. | Every render = GPU cost to the company |
| Vidra (default) | Every render = $0 cost to the company (user's hardware) |
| Vidra (cloud opt-in) | Only premium renders = GPU cost, passed through to user |

This means Vidra can offer a genuinely generous free tier because free users cost almost nothing to serve — object storage and API calls, not GPU hours.

### Critical Design Constraint

The IR and render graph must be **location-agnostic** from day one. A render job must not know or care whether it executes on a laptop or a cloud cluster. Only the execution layer changes. This prevents architectural forks and ensures the transition from local to cloud is a single config change.

### Deterministic Rendering

Every render must be **content-addressable**. Same input IR produces the same output bytes, every time, on any supported hardware. This is enforced through a conformance test suite — a set of reference renders that must produce bit-identical output across all supported platforms (NVIDIA, AMD, Apple Silicon). Determinism unlocks aggressive caching, version diffing, and verifiable builds.

---

## 8. Hybrid Sync Architecture

The hybrid sync model is the core operational model for Vidra. It defines how local work and cloud state stay in sync, how assets flow between environments, and how rendering happens without requiring cloud GPU.

### 8.1 The Sync Model

```
┌──────────────────┐          ┌──────────────────────┐
│   LOCAL MACHINE   │  sync    │    VIDRA CLOUD        │
│                   │ ◄──────► │                       │
│ • vidra.config    │          │ • Project metadata    │
│ • VidraScript     │          │ • Asset manifests     │
│ • Local assets    │          │ • Render receipts     │
│ • Asset cache     │          │ • Shared previews     │
│ • Render output   │          │ • Team state          │
│ • GPU/CPU work    │          │ • Resource library    │
│                   │          │ • Cloud storage (R2)  │
└──────────────────┘          └──────────────────────┘
```

**What syncs to cloud:**
- Project configuration (vidra.config.toml / .ts / .json)
- Asset manifests (hashes + metadata, not the files themselves unless explicitly uploaded)
- Render receipts (proof of render, metadata, output hash)
- Shared previews and final renders (uploaded to object storage)
- Component and template references

**What stays local by default:**
- Source VidraScript files (synced only if user opts in)
- Raw assets (synced only if user explicitly uploads)
- Render cache and intermediate buffers
- GPU/CPU computation

### 8.2 `vidra sync` — Bidirectional Project Sync

One command pulls cloud project state locally or pushes local state to cloud. It handles project metadata, asset manifests, render receipts, team state, and resource references.

```bash
vidra sync              # Bidirectional sync (pull + push)
vidra sync --pull       # Pull cloud state to local
vidra sync --push       # Push local state to cloud
vidra sync --status     # Show what's changed since last sync
```

**Conflict resolution:** Last-write-wins for metadata. Content-addressed deduplication for assets (same hash = same file, no conflict possible). Merge conflicts in VidraScript (if source sync is enabled) surface as standard text diffs.

**Offline workflow:** A user works offline on a plane. They create scenes, render locally, iterate. When they reconnect, `vidra sync` pushes all changes, uploads render receipts, and pulls any team updates. The experience feels seamless regardless of connectivity.

### 8.3 Smart Asset Hydration

When syncing a cloud project locally, assets are **not** downloaded immediately. The sync pulls manifests and thumbnails only. Actual asset files (images, video clips, audio, fonts) are fetched on-demand when the renderer needs them, then cached locally.

```
Project sync: 2.3 MB (manifests + thumbnails)  ← instant
First render: 340 MB (assets pulled on demand)  ← progressive
Subsequent renders: 0 MB (cached locally)       ← instant
```

A project with 2GB of assets syncs in under a second. You only download what you render. Assets are cached in `~/.vidra/cache/` with LRU eviction.

### 8.4 Cloud-Queued, Locally-Executed Render Jobs

The cloud dashboard lets users (or team leads, PMs, automated pipelines) queue render jobs. The local CLI polls for pending jobs assigned to the user, pulls the job spec, renders locally, and uploads results to cloud storage.

```bash
vidra jobs              # List pending render jobs from cloud
vidra jobs --run        # Pull next job, render locally, upload result
vidra jobs --run-all    # Process all pending jobs
vidra jobs --watch      # Continuously poll and execute (daemon mode)
```

This gives teams a cloud-like experience — queue a batch of personalized videos from the dashboard — but the compute is free (user hardware). The cloud never touches a GPU. For users who want fully hands-off cloud rendering, that's the premium upgrade path.

### 8.5 Render Receipts

Every completed render produces a **signed render receipt** — a small JSON payload that uploads to the cloud automatically on next sync.

```json
{
  "receipt_id": "rr_a8f3c9e2",
  "project_id": "proj_vidra_launch",
  "ir_hash": "sha256:e3b0c44298fc1c...",
  "output_hash": "sha256:7f83b1657ff1fc...",
  "output_format": "h264_1080p_30fps",
  "render_duration_ms": 3420,
  "frame_count": 900,
  "hardware": {
    "gpu": "NVIDIA RTX 4070",
    "vram_gb": 12,
    "cpu": "AMD Ryzen 9 7950X",
    "os": "linux_x86_64"
  },
  "vlt_id": "vlt_usr_9x8f2a",
  "timestamp": "2026-02-21T14:32:00Z",
  "signature": "ed25519:..."
}
```

**Render receipts enable:**
- Cloud dashboards and analytics without cloud rendering
- Hardware-aware performance benchmarking across the user base
- Audit trails for enterprise and regulated industries
- Verification that a specific output was produced from a specific IR version
- Usage metering for future plan limits (e.g., free tier: 500 renders/month)

### 8.6 `vidra preview --share` — Instant Cloud Preview from Local

One command renders a low-res preview locally and uploads it to cloud storage with a shareable link.

```bash
vidra preview --share
# → Rendering preview (720p)... done (1.2s)
# → Uploading... done
# → https://share.vidra.dev/p/a8f3c9e2
# → Link copied to clipboard
```

The render is local (free compute). Only the upload costs storage (pennies). This bridges local work and cloud collaboration without requiring cloud rendering.

### 8.7 Resource Upload and Cloud Asset Management

Users can choose to upload assets to their cloud project for team access and backup:

```bash
vidra upload ./assets/hero.mp4       # Upload specific file
vidra upload ./assets/               # Upload directory
vidra assets --list                  # List cloud-stored assets
vidra assets --pull logo.png         # Pull specific cloud asset locally
```

Cloud-stored assets are available to all team members on sync. They also become available for cloud rendering (Layer 3) without needing to transfer them at render time.

### 8.8 Project Configuration

The `vidra.config.toml` (or `.ts` / `.json`) is the single source of truth for project setup. It is checked into version control and synced to cloud.

```toml
[project]
name = "product-launch-q2"
resolution = "1920x1080"
fps = 30
default_format = "h264"

[brand]
kit = "./brand/acme.vidrabrand"

[sync]
enabled = true
auto_sync = true
sync_source = false          # Don't sync VidraScript source to cloud
sync_assets = "on-demand"    # Assets pulled when needed

[render]
target = "local"             # "local" | "cloud"
cloud_fallback = false       # Don't fall back to cloud if local fails
targets = ["16:9", "9:16"]

[telemetry]
level = "identified"         # "anonymous" | "identified" | "diagnostics"

[resources]
registries = ["vidra-commons", "our-team-registry"]
```

---

## 9. Product Pillars

### Pillar 1: Programmable Video Engine

A rendering engine that compiles video definitions into optimized execution graphs, runs on the user's local GPU, and produces deterministic output.

### Pillar 2: Multi-Interface Creation

Users can create video through code (VidraScript / SDK), visual tools (web editor, component playground), and natural language (LLM + MCP). All interfaces compile to the same IR. No interface is second-class.

### Pillar 3: Composition-First Design

Video is built from composable, reusable, parameterized components — not monolithic timelines. Components nest, override, version independently, and are shareable through the marketplace and resource library.

### Pillar 4: Performance as a Feature

Rendering performance is not aspirational — it is specified, measured, profiled, and regression-tested. The platform ships with built-in benchmarking, profiling, and observability tooling.

### Pillar 5: AI-Native Architecture

The system is designed to be controlled by AI agents natively through MCP. The IR is both the developer interface and the AI interface. There is no separate "AI mode."

### Pillar 6: Local Compute, Cloud Coordination

Rendering defaults to user hardware. The cloud coordinates — it never computes unless the user explicitly opts into managed GPU. This keeps costs radically low for both Vidra and its users.

---

## 10. The Vidra Engine — Technical Foundation

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

## 11. The Vidra IR — Video as a Data Structure

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

## 12. VidraScript — A Typed DSL for Video

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
import { LowerThird } from "@vidra-commons/broadcast-kit"

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

## 13. Feature Roadmap — MVP through V3

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
- **CLI:** `vidra init`, `vidra dev`, `vidra render`, `vidra check`, `vidra bench`, `vidra test`, `vidra inspect`, `vidra doctor`
- **VLT:** Vidra License Token for all users (including free), offline-capable authentication and plan enforcement
- **Telemetry:** Tiered telemetry system (anonymous / identified / diagnostics) with full transparency
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
- **`vidra doctor`:** Environment health check (GPU drivers, VRAM, VLT, cache integrity, conformance)

### V2 (Phase 2 — Platform + AI + Ecosystem)

Cloud coordination, LLM-native creation, resource library, collaboration.

- **Hybrid Sync:** `vidra sync` with bidirectional project sync, smart asset hydration, offline reconciliation
- **Render Receipts:** Signed receipts for every local render, auto-uploaded on sync
- **Cloud Dashboard:** Project management, render history, analytics, team overview — no GPU compute
- **Cloud Job Queue:** Queue render jobs from dashboard, execute locally via `vidra jobs --run`
- **`vidra preview --share`:** Local render → instant cloud preview link
- **Cloud Asset Management:** Upload, organize, and share assets across team via cloud storage
- **Vidra Commons:** Community resource library — sounds, video clips, images, textures, LUTs, fonts, inspiration boards
- **`vidra add` for resources:** Install components and raw creative assets from Vidra Commons
- **Curated Starter Kits:** First-party collections (YouTube Intro Kit, Product Launch Kit, Game UI Kit, Social Media Kit)
- **Inspiration Boards:** Curated collections of references, mood boards, motion studies — browsable and searchable
- **License-Aware Asset Management:** Machine-readable licenses on all resources; `vidra licenses` outputs dependency report
- **MCP server:** First-party Model Context Protocol server exposing all Vidra capabilities as LLM tools
- **Conversational storyboarding:** Text-to-storyboard via MCP before rendering
- **Brand Kit:** Persistent brand context (colors, fonts, logos, motion style) injected into MCP
- **`vidra share`:** One-link preview with timestamped commenting and feedback loop
- **Cloud GPU rendering (beta):** Managed render infrastructure for limited beta testers, queue-based, usage-priced
- **Collaboration:** CRDT-based multiplayer editing across code and visual interfaces
- **Marketplace:** Community-published components, templates, effects, and audio
- **AI copilot editor:** Visual editor with inline AI assistance
- **Semantic editing:** Natural language commands applied to the IR
- **Asset intelligence:** Auto-tagging, smart cropping, content-aware layout
- **Render streaming:** Progressive output — video begins playing before render completes
- **Version history:** Full project history with visual diffs between versions
- **Render observability dashboard:** Traces, metrics, GPU profiling per render job
- **Plugin system:** Third-party extensions registered in the IR schema
- **Native AI pipeline hooks:** AI models as first-class render graph nodes with shared GPU memory
- **GitHub integration:** Renders triggered on PR, visual diffs in review, deploy-on-merge
- **Community Challenges:** Weekly/monthly creative challenges with featured showcases
- **`vidra explore`:** Browse trending resources, featured work, and community highlights from the CLI

### V3 (Phase 3 — Ecosystem + Edge + Enterprise)

Open ecosystem, edge runtime, enterprise, cloud GPU general availability.

- **Cloud GPU GA:** Managed cloud rendering available to all paid tiers with usage-based pricing
- **Edge runtime:** WASM-compiled lightweight renderer for edge nodes (Cloudflare Workers, Fastly Compute) — personalized video rendered at the CDN edge in < 100ms
- **Public IR spec:** Open specification for the Vidra IR, enabling third-party tools to read/write Vidra projects
- **Community runtime ports:** Community-maintained renderer implementations for specialized hardware
- **Third-party plugins:** Open plugin API with sandboxed execution
- **Enterprise features:** SSO, audit logs, role-based access, SLA guarantees, dedicated support
- **Machine seat licensing:** VLT bound to N machines per seat (Pro: 3, Team: 5, Enterprise: custom)
- **Live collaboration protocol:** Open CRDT protocol for real-time multi-user editing from any client
- **Broadcast integration:** Live video output to RTMP/SRT for streaming platforms
- **After Effects import:** .aep project file parsing and conversion to Vidra IR
- **Team resource registries:** Private registries for enterprise teams to publish and share internal resources
- **Advanced analytics:** Hardware performance benchmarking across the user base, render cost optimization recommendations

---

## 14. S-Tier Developer Features

These features define Vidra's identity as a developer tool and create defensible differentiation.

### 14.1 Sub-Second Preview (Hot Reload for Video)

When a developer changes a line of VidraScript or adjusts a parameter, the preview updates in under 500ms. Powered by the IR diff engine — only re-render frames that actually changed. This is Vidra's "Vite moment." Every other tool forces you to wait. Vidra doesn't.

### 14.2 `vidra inspect` — X-Ray Vision for Video

A CLI and visual debugger that lets developers see inside any render. Hover over any frame and see the full render tree: which layers are composited, what shaders are running, where time is spent, what the GPU is doing. Click any visual element and see its full lineage back to the VidraScript that produced it. No video tool on earth has this.

### 14.3 Time-Travel Debugging

Every render produces a replayable trace. When something looks wrong at frame 847, you don't re-render — you scrub to that frame in the debugger and step through the render graph execution. Inspect intermediate buffer states, shader outputs, and composition results at any point in the pipeline.

### 14.4 `vidra test` — Visual Regression Testing

Built-in snapshot testing for video. Define key frames or time ranges as test assertions. On every code change, Vidra renders those frames and diffs them pixel-by-pixel against the baseline, with configurable tolerance thresholds. If your brand intro drifts by one pixel after a refactor, CI catches it.

### 14.5 Video Storybook — Component Playground

A local dev server that renders every video component in isolation with adjustable props. Your lower-third component shows up with sliders for duration, color, text length. Your transition component shows up with dropdowns for easing curves. Designers and developers share this as a living catalog of everything the system can produce.

### 14.6 `vidra bench` — Performance Profiling

One command benchmarks your project across resolutions, durations, and hardware profiles. Structured report showing render time per scene, GPU memory peaks, asset decode bottlenecks, and frame-over-frame cost distribution. Flags regressions against your last run. Commit the baseline to git; CI tells you when a PR makes renders slower.

### 14.7 Render Observability — Traces, Metrics, Profiling

Every render job emits structured traces: which frames were slow, which assets took longest to decode, where GPU utilization dropped, memory high-water marks. Exposed through a dashboard and API. This is how you earn trust from engineering teams running Vidra in production.

### 14.8 Render Streaming — Progressive Output

Don't make users wait for a full render. Stream encoded frames as they're produced using chunked encoding and out-of-order frame assembly. A 60-second video starts playing back within seconds of the render starting.

### 14.9 Live Collaboration Protocol

A CRDT-based protocol for real-time multiplayer editing at the IR level. One person in VS Code, another in the visual editor — both see each other's changes in real time. The protocol is open so third-party editors can plug in.

### 14.10 Escape Hatch Interop Layer

One-command import from FFmpeg filter graphs, Remotion projects, Lottie/Rive animations, Apple Motion templates. One-command export to ProRes, H.264/5, VP9, AV1, GIF, image sequences, MPEG-DASH/HLS. The exit door is what gets people to walk in.

### 14.11 `vidra doctor` — Environment Health Check

One command validates the entire development environment: GPU driver version and compatibility, available VRAM and RAM, VLT validity and expiry, local asset cache integrity, conformance test pass/fail status, network connectivity to Vidra Cloud, and installed CLI/SDK version. When a user files a bug report, `vidra doctor` output provides everything needed for diagnosis.

```bash
$ vidra doctor

  ✓ GPU: NVIDIA RTX 4070 (12GB VRAM) — driver 550.54 ✓
  ✓ Renderer: wgpu 0.19 — Vulkan backend
  ✓ VRAM available: 10.2 GB
  ✓ RAM available: 24.1 GB
  ✓ VLT: valid — expires 2026-03-15 — plan: pro
  ✓ Asset cache: 1.2 GB — integrity OK
  ✓ Conformance: 147/147 tests passed
  ✓ Cloud sync: connected — last sync 4 minutes ago
  ✓ CLI version: 1.4.2 (latest)

  All systems nominal.
```

---

## 15. Non-Developer Experience — LLM-Native Creation via MCP

### The Core Idea

Anyone with access to an LLM can create, edit, and render video through natural conversation. The LLM writes VidraScript so the human doesn't have to. The IR is both the developer interface and the AI interface. There is no separate "AI mode."

### 15.1 Vidra MCP Server

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
| `vidra.add_resource` | Pull a resource from Vidra Commons |
| `vidra.list_templates` | Browse available templates and components |
| `vidra.list_resources` | Search the resource library |
| `vidra.share` | Generate a shareable preview link |

### Example Workflow

**User (in Claude):** *"Make me a 30-second product launch video for my new sneaker brand. The vibe is dark, minimal, with quick cuts."*

**Claude** calls `vidra.storyboard` → generates a visual storyboard with 6 key frames. User reviews, says *"Love frames 1-4, frame 5 needs more energy, cut frame 6."* Claude calls `vidra.create_project`, `vidra.add_scene` (×5), `vidra.add_resource` (pulls cinematic whoosh sounds and grain textures from Vidra Commons), `vidra.set_style`, `vidra.render_preview`. User sees the preview, says *"Make the text bigger and add a bass-heavy sound effect on the transitions."* Claude calls `vidra.edit_layer` and `vidra.add_resource` to make surgical edits.

The user never sees code. The user never sees a timeline. The output is professional.

### 15.2 Conversational Storyboarding

Before rendering, the MCP server generates a lightweight storyboard — a grid of static key frames with timing annotations — from a text description. Users iterate on the concept before a single frame is rendered. This dramatically reduces the cost of exploration and makes the LLM interaction feel collaborative.

### 15.3 Brand Kit as Context

Users define a brand kit (colors, fonts, logos, motion style, audio signatures) that is injected into the MCP server's context. Every video the LLM creates automatically adheres to brand guidelines without the user specifying them each time. Set it up once; every video is on-brand by default.

### 15.4 `vidra share` — One-Link Preview and Feedback

After rendering, the user gets a shareable link with an embedded player and a timestamped comment layer. Reviewers leave feedback directly on the video. The LLM consumes that feedback through MCP and makes edits. The entire review cycle — create, share, get feedback, revise — happens without anyone touching an editor.

---

## 16. Game Developer Pipeline

### The Problem

Game developers need high volumes of 2D animated assets: UI animations, cutscene segments, promotional trailers, in-game video textures, loading screen animations, ability/spell effects. The current pipeline involves After Effects → manual export → manual format conversion → import to engine → realize it looks wrong → repeat.

### 16.1 Sprite Sheet and Texture Sequence Export

Render any Vidra composition to a packed sprite sheet or numbered image sequence optimized for game engines. Direct export formats for Unity (sprite atlas), Unreal (flipbook texture), and Godot (AnimatedSprite2D). A game dev writes VidraScript for a fire effect and exports it directly into their engine's asset pipeline.

### 16.2 Parameterized Asset Variants

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

### 16.3 Procedural Animation Nodes

Built-in GPU-accelerated nodes for common game asset patterns: particle systems, procedural noise (Perlin, simplex, Worley), sine-wave distortions, glow and bloom, chromatic aberration, screen shake, glitch effects, dissolve transitions, outline/silhouette rendering. These compose natively with all other layers and effects.

### 16.4 Engine-Aware Preview

A preview mode that simulates how the asset will appear inside a target game engine — with the correct color space, compression artifacts, mip levels, and target frame rate. What you see in Vidra preview matches what you get in-engine.

---

## 17. Performance Guarantees & SLAs

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

## 18. Composition Model — Video as Components

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
- **Publishing** — components are publishable to the Vidra Marketplace and Vidra Commons

---

## 19. Multi-Target Responsive Output

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

## 20. Vidra License Token (VLT) & Telemetry

### 20.1 Vidra License Token (VLT)

Every user — including free-tier users — authenticates with a Vidra License Token. The VLT is a signed, offline-capable credential that serves three purposes simultaneously:

| Purpose | What it does |
|---|---|
| **Identity** | Who is this user? Ties activity to an account. |
| **Entitlement** | What are they allowed to do? Plan tier, feature flags, rate limits. |
| **Telemetry anchor** | What are they doing? Render counts, usage patterns (per telemetry tier). |

### VLT Design

The VLT is a signed JWT-like token with embedded claims:

```json
{
  "vlt_id": "vlt_usr_9x8f2a",
  "user_id": "usr_4k2m8n",
  "plan": "pro",
  "features": ["cloud_sync", "share", "brand_kit", "commons_premium"],
  "limits": {
    "renders_per_month": null,
    "cloud_renders_per_month": 100,
    "machines": 3,
    "team_members": null
  },
  "issued_at": "2026-02-01T00:00:00Z",
  "expires_at": "2026-03-03T00:00:00Z",
  "signature": "ed25519:..."
}
```

### Offline Behavior

The CLI validates the VLT locally without network access. The token carries all the information needed for plan enforcement. When the user syncs or connects to the internet, a fresh VLT is issued with updated claims.

| Scenario | Behavior |
|---|---|
| Online | VLT refreshed on sync, always current |
| Offline, VLT valid | Full local functionality, no degradation |
| Offline, VLT expired | 7-day grace period with warning |
| Offline, VLT expired + grace exhausted | Local rendering still works; cloud features disabled |

**Critical principle:** Local rendering never stops working. The VLT gates cloud features and telemetry, not the ability to render on your own machine. A user who never connects to the internet can still render video forever. The VLT is about coordination, not control.

### Machine Seat Licensing (V3)

For paid tiers, the VLT is bound to N machines per seat via a hardware fingerprint included in render receipts.

| Plan | Machines per seat |
|---|---|
| Free | Unlimited (local-only features) |
| Pro | 3 |
| Team | 5 per seat |
| Enterprise | Custom |

### 20.2 Telemetry

Vidra collects telemetry in three tiers. Users choose their tier in `vidra.config.toml` or during `vidra init`. The default is `identified`.

| Tier | What's collected | Who sees it |
|---|---|---|
| **Anonymous** | Render count, duration, resolution, error rates, engine version. No user identity, no content. | Vidra team (aggregate only) |
| **Identified** | Anonymous data + tied to VLT ID. Used for billing, plan limits, and per-user dashboards. | Vidra team + user's own dashboard |
| **Diagnostics** | Identified data + GPU profiling, crash dumps, render traces. Opt-in for debugging. | Vidra team (support cases) |

### Telemetry Transparency

Vidra publishes a **Telemetry Specification** document describing exactly what is collected at each tier, how it's stored, and how long it's retained.

```bash
vidra telemetry show          # Display what data is being collected
vidra telemetry set anonymous # Change telemetry tier
vidra telemetry export        # Export all collected data (GDPR)
vidra telemetry delete        # Request deletion of all telemetry data
```

Users can opt out of anonymous telemetry entirely. Developers hate hidden telemetry. They respect transparent telemetry.

### API Keys for Integrations

In addition to the VLT (which is a user/machine credential), Vidra issues **API keys** for programmatic access — CI/CD pipelines, automated renders, MCP server connections, and third-party integrations. API keys are scoped, rotatable, and rate-limited per plan tier.

```bash
vidra auth create-key --name "ci-pipeline" --scope "render,sync"
# → vk_live_7x9m2p4a...

vidra auth list-keys
vidra auth revoke-key vk_live_7x9m2p4a
```

---

## 21. Vidra Commons — Resource Library & Ecosystem

Vidra Commons is the shared creative resource layer — an open library of sounds, video clips, images, textures, LUTs, fonts, inspiration boards, components, and templates. It is to creative assets what npm is to code packages.

### 21.1 What's in Vidra Commons

| Resource Type | Examples |
|---|---|
| **Audio** | Sound effects, ambient tracks, music loops, transition whooshes |
| **Video clips** | Stock footage, backgrounds, overlays, light leaks, bokeh |
| **Images** | Textures, gradients, patterns, grain overlays, backgrounds |
| **Fonts** | Open-source typefaces curated for video |
| **LUTs** | Color grading presets |
| **Swatches** | Color palettes, brand color collections |
| **Components** | Reusable VidraScript components (lower-thirds, intros, transitions) |
| **Templates** | Full video compositions with parameterized placeholders |
| **Procedural nodes** | Custom effects, generators, and GPU shaders |
| **Inspiration boards** | Curated collections of references, mood boards, motion studies |

### 21.2 Content-Addressed, Versioned, Composable

Every resource in Vidra Commons is content-addressed (hash-based), versioned, and composable. A VidraScript file can reference a resource by hash, and it resolves deterministically regardless of where or when the project is rendered.

```vidra
import { cinematic_whoosh } from "@vidra-commons/sfx-pack"
import { film_grain_16mm } from "@vidra-commons/textures-analog"

scene("reveal", 3s) {
  layer("grain") {
    image(film_grain_16mm, blend: overlay, opacity: 0.3)
  }
  audio(cinematic_whoosh, at: 0.5s)
}
```

### 21.3 `vidra add` for Resources

The existing `vidra add` command handles both components and raw resources:

```bash
vidra add @vidra-commons/cinematic-whoosh      # Add a sound effect
vidra add @vidra-commons/grain-overlays         # Add a texture pack
vidra add @vidra-commons/broadcast-kit          # Add a component library
vidra add @vidra-commons/youtube-starter-kit    # Add an entire starter kit

vidra search "whoosh transition"                # Search Vidra Commons
vidra search --type audio "ambient rain"        # Search by type
vidra search --type inspiration "cyberpunk"     # Search inspiration boards
```

Resources land in the local asset cache (`~/.vidra/cache/`) and are referenced in the project manifest. On sync, manifests reference content hashes — actual files are deduplicated across all users in cloud storage.

### 21.4 Curated Starter Kits

First-party collections assembled by the Vidra team. Each kit includes templates, components, sounds, fonts, and complete example projects that work out of the box.

| Kit | Contents | Target User |
|---|---|---|
| **YouTube Intro Kit** | 5 intro templates, lower-thirds, subscribe animations, transition pack, royalty-free music | Content creators |
| **Product Launch Kit** | Hero reveal, feature callouts, pricing cards, CTA animations, corporate fonts | Marketing teams |
| **Game UI Kit** | Health bars, damage popups, XP animations, loot reveals, pixel fonts | Game developers |
| **Social Media Kit** | Story templates, reel transitions, text animations, trending audio stubs | Social media managers |
| **Corporate Kit** | Branded intros, data visualizations, talking-head frames, quarterly report templates | Enterprise teams |
| **Cinematic Kit** | Film grain, light leaks, letterbox overlays, cinematic LUTs, orchestral stingers | Filmmakers |

```bash
vidra init my-project --kit youtube
# → Project created with YouTube Intro Kit
# → Includes: 5 templates, 12 components, 8 audio files, 3 fonts
# → Run `vidra dev` to start editing
```

This solves the empty canvas problem that kills adoption.

### 21.5 Inspiration Boards

Users can publish and browse curated collections of references — mood boards, style references, motion studies — tagged and searchable. These are not functional assets; they are creative context.

When combined with the MCP flow, a non-dev can say *"I want something like this inspiration board"* and the LLM uses it as creative direction. This turns the resource library into a creative discovery tool, not just an asset store.

```bash
vidra explore                          # Browse trending content
vidra explore --featured               # Staff picks and highlighted work
vidra explore --challenges             # Current community challenges
vidra explore --boards "minimalist"    # Browse inspiration boards
```

### 21.6 Community Challenges and Showcases

Weekly or monthly creative challenges where users submit renders made with Vidra. Featured work appears on the Vidra Commons homepage, in the CLI via `vidra explore`, and in marketing materials. The best submissions get promoted to starter kits and featured collections.

This builds community, generates content for marketing, and populates the library with real-world examples that inspire new users.

### 21.7 License-Aware Asset Management

Every resource in Vidra Commons carries a machine-readable license:

| License | Usage |
|---|---|
| CC0 | Public domain, no restrictions |
| CC-BY | Free with attribution |
| CC-BY-SA | Free with attribution, share-alike |
| MIT | Permissive open source |
| Vidra Commercial | Free for Vidra projects, no redistribution outside Vidra |
| Premium | Requires paid plan or per-asset purchase |

The CLI and cloud dashboard show license requirements for your project's dependency tree:

```bash
vidra licenses
# → 14 resources used
# → 8 CC0 (no requirements)
# → 4 CC-BY (attribution required — see ATTRIBUTION.md)
# → 2 Premium (covered by Pro plan)
# → ✓ All licenses satisfied
```

This matters for agencies and enterprise users who need legal clarity on every asset they ship.

### 21.8 Publishing to Vidra Commons

Any user can publish resources:

```bash
vidra publish --type component ./my-lower-third/
vidra publish --type audio ./whoosh.wav --license cc0
vidra publish --type inspiration ./mood-board/ --tags "cyberpunk,neon,dark"
```

All submissions pass automated review: renders without errors (for components), metadata is complete, license is specified, content policy check passes.

---

## 22. Marketplace & Ecosystem

The Marketplace is the commercial layer on top of Vidra Commons. While Commons includes free and open resources, the Marketplace supports premium paid content.

### What Gets Published

- **Premium components** — professional-grade video building blocks
- **Premium templates** — full video compositions with advanced features
- **Premium procedural nodes** — custom effects and GPU shaders
- **Brand kits** — curated style packages
- **Audio packs** — licensed music and sound effects
- **Plugins** — IR extensions and custom render nodes

### Revenue Model

Marketplace operates on a revenue share model. Free resources in Vidra Commons drive ecosystem growth; premium Marketplace content drives creator revenue and platform margin.

### Quality Standards

All Marketplace submissions pass automated and manual review: renders without errors, meets conformance tests, documentation is present, props are typed and documented, preview renders are provided.

---

## 23. The Unified UX/DX Story

### For Developers

Vidra is the Rust-powered, GPU-accelerated video engine with the best CLI tooling in any creative domain. You write VidraScript or use the TypeScript/Python SDK. You get instant previews with hot-reload. You have real debugging tools — inspect, time-travel, profiling. You ship with confidence via visual regression tests in CI. It's local-first, you own your pipeline, and it's faster than anything else because it's built on a real rendering engine. Starter kits and Vidra Commons mean you're never starting from scratch.

### For Non-Developers

Vidra is the thing your AI assistant uses to make videos for you. You describe what you want in natural language, review storyboards and previews, give feedback in conversation, and professional-quality video comes out. You never see code. You never see a timeline. You never feel limited. Your brand kit ensures everything is consistent. The resource library provides sounds, textures, and inspiration at your fingertips.

### For Game Developers

Vidra is the programmable motion graphics pipeline that speaks your language — parameters, nodes, batch export, engine-native formats. It replaces the After Effects → export → import → realize-it's-wrong loop with a code-defined, CI-integrated asset pipeline that produces exactly what your engine needs.

### For Teams

Vidra is the collaborative platform where developers build the system, designers curate the component library, marketers generate content through conversation, and everything stays on-brand and version-controlled. The visual editor and the code editor are two views of the same IR. The cloud dashboard coordinates without ever billing you for GPU time unless you opt in.

---

## 24. CLI & Developer Workflow

### Commands

| Command | Description |
|---|---|
| `vidra init <name>` | Scaffold a new project with sensible defaults |
| `vidra init --kit <kit>` | Scaffold with a starter kit |
| `vidra dev` | Start local preview server with hot-reload |
| `vidra render` | Render locally, full quality |
| `vidra render --cloud` | Send to managed cloud render (V2+, premium) |
| `vidra render --targets` | Multi-target responsive render |
| `vidra check` | Static analysis, type checking, linting |
| `vidra fmt` | Auto-format VidraScript files |
| `vidra test` | Run visual regression snapshot tests |
| `vidra test --update` | Update snapshot baselines |
| `vidra bench` | Performance benchmark with regression detection |
| `vidra inspect` | Open visual render debugger |
| `vidra inspect --frame N` | Jump to specific frame in debugger |
| `vidra doctor` | Environment health check |
| `vidra sync` | Bidirectional project sync with cloud |
| `vidra sync --status` | Show pending sync changes |
| `vidra jobs` | List pending cloud render jobs |
| `vidra jobs --run` | Execute next queued job locally |
| `vidra jobs --watch` | Daemon mode: poll and execute jobs |
| `vidra preview --share` | Render locally, upload preview, get link |
| `vidra share` | Generate shareable link for latest render |
| `vidra upload` | Upload assets to cloud project storage |
| `vidra assets` | Manage cloud-stored assets |
| `vidra add <package>` | Install component or resource from Commons |
| `vidra search <query>` | Search Vidra Commons |
| `vidra explore` | Browse trending resources and featured work |
| `vidra publish` | Publish to Vidra Commons / Marketplace |
| `vidra licenses` | Show license report for all project dependencies |
| `vidra auth login` | Authenticate and obtain VLT |
| `vidra auth create-key` | Create scoped API key |
| `vidra auth list-keys` | List active API keys |
| `vidra auth revoke-key` | Revoke an API key |
| `vidra telemetry show` | Display current telemetry settings and data |
| `vidra telemetry set` | Change telemetry tier |
| `vidra export --spritesheet` | Export as packed sprite sheet |
| `vidra export --sequence` | Export as numbered image sequence |

### Zero-to-First-Render

```bash
# Install (via shell script, npm, or brew)
curl -fsSL https://vidra.dev/install.sh | sh

# Authenticate (free account)
vidra auth login
# → Browser opens, sign in, VLT stored locally

# Create project with a starter kit
vidra init my-first-video --kit youtube
cd my-first-video

# Start dev server — preview opens in browser
vidra dev

# Edit main.vidra in your editor of choice
# Preview updates in < 500ms on save

# Render final output
vidra render
# → output/my-first-video.mp4 (rendered in seconds, on your GPU)

# Share with someone
vidra preview --share
# → https://share.vidra.dev/p/a8f3c9e2
```

**Target: under 60 seconds from install to seeing your first rendered frame.**

### GitHub-Native Workflow (V2)

- Renders triggered on pull request via API key
- Visual diffs embedded in PR review (frame-by-frame comparison)
- Deploy-on-merge to CDN for hosting
- Video becomes a CI/CD artifact like any other build output

---

## 25. Competitive Landscape

| | Vidra | Remotion | FFmpeg | After Effects | Runway |
|---|---|---|---|---|---|
| **Rendering** | Rust + GPU | Headless Chrome | CLI pipeline | Local app | Cloud |
| **Performance** | 10-50x browser | Baseline | Fast but no composition | Slow render | Queue-based |
| **Programmability** | Full (DSL + SDK + API) | SDK only | CLI flags | ExtendScript (limited) | Prompt only |
| **Local-first** | Yes | Yes | Yes | Yes | No |
| **Cloud cost model** | $0 default (local render) | N/A | N/A | N/A | Every render = cost |
| **AI-native (MCP)** | Yes | No | No | No | Partial |
| **Composability** | Component model | React components | None | Precomps (limited) | None |
| **Testing/CI** | Built-in | Manual | Manual | None | None |
| **Debugging** | Inspect + time-travel | Browser DevTools | None | Limited | None |
| **Resource library** | Vidra Commons | None | None | Adobe Stock (separate) | None |
| **Game dev export** | Native | Manual | Manual | Manual | No |
| **Multi-target** | Layout rules | Manual | Manual | Manual | No |
| **Deterministic** | Guaranteed | No (browser variance) | Mostly | No | No |
| **Offline licensing** | VLT (offline-capable) | Open source | Open source | Adobe CC (online) | Cloud only |

### Positioning Against Remotion

Remotion proved the market for code-defined video. Vidra takes the same distribution strategy (local-first, developer-friendly, code-driven) and pairs it with a fundamentally superior engine. Remotion is limited by the browser rendering model — it's clever but slow, can't do real GPU-accelerated effects, and has no video-native IR. Vidra is to Remotion what Node.js was to PHP: same developer-friendly ethos, completely different runtime that unlocks things the old approach cannot do.

---

## 26. Monetization Strategy

### The Cost Model Advantage

Because rendering defaults to user hardware, Vidra's per-user cost is near-zero for the majority of users. This enables a genuinely generous free tier and shifts the monetization to coordination, collaboration, and premium cloud services.

### Phase 1 — Free Engine, VLT for Everyone

The engine (Layer 1) is free and source-available. Every user gets a VLT. Revenue: $0. Cost: minimal (auth service + telemetry ingestion).

### Phase 2 — Platform Freemium

The coordination platform (Layer 2) launches with a freemium model.

| Tier | Price | Includes |
|---|---|---|
| **Free** | $0 | Local engine, VLT, 500 render receipts/month, 5 shared previews/month, Vidra Commons (free resources), 1 starter kit, anonymous telemetry |
| **Pro** | $29/month | Unlimited render receipts, unlimited sharing, cloud sync, brand kits, version history, Commons premium resources, all starter kits, 3 machine seats, priority support |
| **Team** | $79/seat/month | Everything in Pro + collaboration, shared libraries, team dashboard, team analytics, cloud job queue, 5 machine seats per user, role management |

### Phase 2+ — Cloud GPU (Beta → GA)

Managed cloud rendering (Layer 3) is initially available to beta testers, then to paid tiers.

| Resolution | Price |
|---|---|
| 720p | $0.005 / render-second |
| 1080p | $0.01 / render-second |
| 4K | $0.04 / render-second |

**Included cloud renders by plan:**
| Plan | Included cloud render-seconds/month |
|---|---|
| Free | 0 (local only) |
| Pro | 100 |
| Team | 500 per seat |
| Enterprise | Custom committed volume |

Volume discounts and committed-use pricing for enterprise.

### Phase 3 — Marketplace Revenue Share

Platform takes 20% of premium component and resource sales. Creators retain 80%.

### Phase 3+ — Enterprise

| Tier | Price | Includes |
|---|---|---|
| **Enterprise** | Custom | SSO, audit logs, SLA guarantees, dedicated support, private resource registry, custom machine seat limits, committed cloud render volume, on-premise deployment option |

---

## 27. Risks & Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **Rendering engine complexity** | High | Incremental delivery: single-threaded first, GPU second, distributed third. Conformance test suite gates every release. |
| **GPU compatibility across hardware** | High | wgpu abstraction layer covers Vulkan/Metal/DX12. Conformance tests run on all target hardware. CPU fallback for CI environments. |
| **Deterministic rendering variance** | High | Content-addressable output verified by conformance suite. Bit-identical rendering is a hard requirement, not a goal. |
| **Encoding performance** | Medium | FFmpeg bindings for broad support; invest in native AV1 encoder for performance-critical paths. |
| **Market education** | Medium | Developer advocacy, public benchmarks, open-source components. Lead with "it's like Remotion but 10-50x faster" narrative. |
| **Local-first → no recurring revenue** | Medium | Platform layer (sync, sharing, brand kits, Commons) provides natural cloud value. Cloud GPU is premium, not default. VLT enables usage tracking from day one. |
| **Remotion entrenchment** | Medium | Performance differentiation is measurable and dramatic. Import tool eases migration. Target use cases Remotion can't serve (GPU effects, game assets, AI pipeline hooks). |
| **LLM/MCP adoption uncertainty** | Low | MCP is additive to the core developer product. If LLM adoption is slower than expected, the developer product stands on its own. |
| **Becoming "just another Remotion"** | Medium | VidraScript DSL, inspect/debug tooling, game dev pipeline, resource library, and AI-native architecture create clear category separation. |
| **Resource library quality** | Medium | Curated first-party starter kits set quality bar. Automated review for all submissions. Community voting surfaces best content. |
| **VLT/telemetry developer pushback** | Medium | Radical transparency (published spec, `vidra telemetry show`, opt-out). Local rendering never gated by VLT. Grace periods are generous. |
| **Free tier abuse** | Low | Render receipts provide usage visibility. Rate limits in VLT claims. Generous limits make abuse unlikely for legitimate users. |

---

## 28. Phased Roadmap & Milestones

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
- [ ] VLT system — free-tier tokens for all users
- [ ] Tiered telemetry system with transparency tooling
- [ ] API key generation for CI/CD
- [ ] `vidra init`, `vidra dev`, `vidra check`, `vidra fmt`
- [ ] `vidra test` — visual regression testing
- [ ] `vidra bench` — performance profiling
- [ ] `vidra inspect` — visual debugger
- [ ] `vidra doctor` — environment health check
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

### Phase 1.5 — Platform + Commons Soft Launch (Months 8-12)

**Goal:** Introduce cloud coordination, resource library, validate monetization.

- [ ] `vidra sync` — bidirectional project sync
- [ ] Smart asset hydration (manifest-first, on-demand fetch)
- [ ] Render receipts — auto-generated, auto-uploaded
- [ ] Cloud dashboard — project overview, render history, analytics
- [ ] `vidra preview --share` — instant shareable preview from local render
- [ ] Cloud asset management — upload, organize, share
- [ ] Vidra Commons — initial resource library (first-party curated)
- [ ] `vidra add` for resources
- [ ] `vidra search` and `vidra explore`
- [ ] 3 curated starter kits (YouTube, Product Launch, Game UI)
- [ ] License-aware asset management (`vidra licenses`)
- [ ] Brand kit system
- [ ] Version history
- [ ] Pro tier launch ($29/month)

**Milestone:** 500 paying Pro users. $15K MRR. 5,000 resources in Vidra Commons.

### Phase 2 — AI, Cloud Jobs, Full Ecosystem (Months 10-18)

**Goal:** Non-developers can create video. Teams adopt at scale. Resource ecosystem thrives.

- [ ] Cloud job queue — queue from dashboard, execute locally
- [ ] `vidra jobs` — pull and execute queued render jobs
- [ ] Vidra MCP server (first-party)
- [ ] Conversational storyboarding via MCP
- [ ] Cloud GPU rendering (limited beta, queue-based)
- [ ] CRDT-based collaboration protocol
- [ ] Community publishing to Vidra Commons
- [ ] Inspiration boards
- [ ] Community challenges and featured showcases
- [ ] All 6 curated starter kits
- [ ] Marketplace (premium content layer)
- [ ] AI copilot in visual editor
- [ ] Native AI pipeline hooks (models as render graph nodes)
- [ ] Render streaming (progressive output)
- [ ] Render observability dashboard
- [ ] Plugin system (third-party IR extensions)
- [ ] GitHub Actions integration
- [ ] Python SDK
- [ ] Team tier launch ($79/seat/month)

**Milestone:** 10,000 weekly active developers. 5,000 MCP-driven renders per day. 50,000 resources in Vidra Commons. $100K MRR.

### Phase 3 — Ecosystem, Edge, Enterprise (Months 18-30)

**Goal:** Vidra is default video infrastructure. Open ecosystem. Sustainable business.

- [ ] Cloud GPU general availability (usage-based pricing for all paid tiers)
- [ ] Edge runtime (WASM-compiled renderer for CDN edge)
- [ ] Public IR specification
- [ ] Open collaboration protocol
- [ ] Machine seat licensing enforcement
- [ ] Enterprise tier (SSO, audit, SLA, private registries)
- [ ] After Effects .aep import
- [ ] Broadcast output (RTMP/SRT)
- [ ] Community runtime ports
- [ ] Third-party plugin sandbox
- [ ] Team resource registries (private)
- [ ] Advanced analytics and optimization recommendations

**Milestone:** 50,000 weekly active developers. 1M+ daily render jobs. 200,000+ resources in Commons. $1M MRR.

---

## 29. Positioning & Brand

### Positioning Statement

Vidra is not a video editor. Vidra is not an AI video generator. Vidra is not a cloud render farm.

**Vidra is video infrastructure** — the programmable, local-first, AI-native runtime that makes video as composable, testable, and deployable as software. Your GPU does the work. Our platform coordinates.

### Tagline

> **"One engine. Every interface. Any scale."**

### Alternative Taglines

- "Video, compiled."
- "The runtime for video."
- "Build video like software."
- "The API for motion."
- "Your GPU. Our platform."

### Narrative Arc

1. **For developers:** "You wouldn't build a web app by manually dragging divs around a screen. Why do you build video that way? Vidra makes video programmable — and it runs on your machine."
2. **For non-developers:** "Describe the video you want. Your AI assistant builds it. You review and refine through conversation. That's it."
3. **For the industry:** "The web got its infrastructure layer decades ago. Video never did. Vidra is that layer — and it doesn't need our servers to run."

---

## 30. Appendix

### A. Glossary

| Term | Definition |
|---|---|
| **Vidra IR** | Intermediate Representation — the canonical, queryable scene graph that represents a video |
| **VidraScript** | Vidra's typed domain-specific language for defining video compositions |
| **VLT** | Vidra License Token — signed, offline-capable credential for identity, entitlement, and telemetry |
| **Render Graph** | The DAG of GPU/CPU operations compiled from the IR for a given frame range |
| **Render Receipt** | Signed proof of a completed render, including IR hash, output hash, hardware info, and duration |
| **MCP** | Model Context Protocol — the standard for exposing tools to LLMs |
| **Component** | A reusable, parameterized video building block with typed props |
| **Brand Kit** | A persistent set of brand assets and style rules (colors, fonts, logos, motion) |
| **Vidra Commons** | The shared resource library — sounds, images, videos, textures, components, templates, inspiration |
| **Conformance Suite** | A set of reference renders used to verify deterministic output across hardware |
| **Hot-Reload** | Sub-second preview updates when source code changes |
| **Smart Asset Hydration** | On-demand asset fetching — download only what the renderer needs, when it needs it |
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

### D. Telemetry Data Specification (Summary)

| Data Point | Anonymous | Identified | Diagnostics |
|---|---|---|---|
| Render count | ✓ | ✓ | ✓ |
| Render duration | ✓ | ✓ | ✓ |
| Output resolution | ✓ | ✓ | ✓ |
| Engine version | ✓ | ✓ | ✓ |
| Error type (no stack trace) | ✓ | ✓ | ✓ |
| OS / GPU model | ✓ | ✓ | ✓ |
| User ID (VLT) | | ✓ | ✓ |
| Project ID | | ✓ | ✓ |
| Feature usage | | ✓ | ✓ |
| GPU frame profiling | | | ✓ |
| Full stack traces | | | ✓ |
| Render graph dumps | | | ✓ |

Full specification published at `docs.vidra.dev/telemetry`.

### E. Project Configuration Reference

```toml
# vidra.config.toml — full reference

[project]
name = "my-project"
resolution = "1920x1080"
fps = 30
default_format = "h264"

[brand]
kit = "./brand/my-brand.vidrabrand"

[sync]
enabled = true
auto_sync = true
sync_source = false
sync_assets = "on-demand"     # "none" | "on-demand" | "all"

[render]
target = "local"              # "local" | "cloud"
cloud_fallback = false
targets = ["16:9"]

[telemetry]
level = "identified"          # "anonymous" | "identified" | "diagnostics" | "off"

[resources]
registries = ["vidra-commons"]
cache_dir = "~/.vidra/cache"
cache_max_gb = 10

[auth]
vlt_path = "~/.vidra/vlt.token"
```

---

*This document is a living artifact. It will evolve as we build, learn, and ship. The principles — local-first, performance-first, composable, AI-native, cloud-coordinated — do not change. Everything else is subject to iteration.*

**— The Vidra Team**
