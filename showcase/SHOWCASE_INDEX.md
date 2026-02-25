# Vidra Kitchen Sink Showcase

Last updated: 2026-02-24

This package is a curated, investor/demo-safe showcase of what is currently available in this repo.

## 1) React Vite Showcase App Status
- Existing app: [packages/vidra-examples](packages/vidra-examples)
- Core implementation: [packages/vidra-examples/src/App.tsx](packages/vidra-examples/src/App.tsx)

Current app includes:
- VidraScript editor + Monaco language support
- SDK code mode
- Interactive player controls
- Asset loading flow
- Creative scene/layer editing surface
- AI-assisted code generation panel

## 2) Curated Video Set (Use This)
Primary investor/demo-safe set is in [showcase/videos_curated](showcase/videos_curated).

### Curated scenarios
- Anchor/centering proof: [showcase/videos_curated/anchor_centered.mp4](showcase/videos_curated/anchor_centered.mp4)
- Audio muxing proof (audible): [showcase/videos_curated/audio_muxing_verified.mp4](showcase/videos_curated/audio_muxing_verified.mp4)
- Image decode robustness proof: [showcase/videos_curated/image_decode_fixed.mp4](showcase/videos_curated/image_decode_fixed.mp4)
- App flow sample: [showcase/videos_curated/app_flow.mp4](showcase/videos_curated/app_flow.mp4)
- Brand flow sample: [showcase/videos_curated/brand_flow.mp4](showcase/videos_curated/brand_flow.mp4)
- Multi-target 16:9: [showcase/videos_curated/multi_target_16x9.mp4](showcase/videos_curated/multi_target_16x9.mp4)
- Multi-target 9:16: [showcase/videos_curated/multi_target_9x16.mp4](showcase/videos_curated/multi_target_9x16.mp4)

## 3) Legacy Set (Do Not Use for Investor Demo)
Legacy copied outputs remain in [showcase/videos](showcase/videos) for historical reference only.
Known defects reported by user testing include:
- off-centering/misalignment in several files
- poor text quality in 4K bench sample
- low-value block-like visuals in legacy demo output

## 4) Reproducibility
- Render script: [showcase/render_showcase.sh](showcase/render_showcase.sh)
- Manifest: [showcase/manifest.json](showcase/manifest.json)

The script now assembles the curated set first, keeps legacy outputs separate, and performs an audio-stream check on the curated muxing proof.

## 5) Honest Claim Boundaries
- These videos are examples from repository test/demo assets.
- They are not guarantees of identical performance on all hardware.
- Any external claim should reference [bench_amrs.md](bench_amrs.md) and [investor.md](investor.md).
