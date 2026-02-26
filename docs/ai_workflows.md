# AI & the Model Context Protocol (MCP)

Vidra is designed to be fully programmable, which naturally extends to being entirely promptable by Large Language Models. 

Instead of an AI trying to write a complex, custom Adobe After Effects script or stitching obscure Python image libraries, Vidra provides an **MCP Server** out of the box.

## The Vidra MCP Server

The **Model Context Protocol (MCP)** gives any AI assistant (like Claude Desktop, cursor, or independent agents) a standardized way to read and write context. Vidra implements a full MCP server mapped directly to its internal engine API.

```bash
vidra mcp
# or without installation: bunx @sansavision/vidra@latest mcp
```

### How to Connect (Claude Desktop Example)

You do not run this command manually and type into it. Instead, you register it with your AI client (like Claude Desktop or Cursor). The AI will run the server in the background and talk to it via JSON-RPC over `stdio`.

To add Vidra to Claude Desktop, open your MCP configuration file (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS) and add:

```json
{
  "mcpServers": {
    "vidra": {
      "command": "bunx",
      "args": [
        "--bun",
        "@sansavision/vidra@latest",
        "mcp"
      ]
    }
  }
}
```

Restart Claude, and it will now automatically understand the `vidra` tools.

### What to ask your LLM

You can now use conversational prompts to generate and edit video:
* *"Create a new Vidra project in the `promo_video` folder. Make it 1080p and 60fps."*
* *"Change the `hero_text` layer's font to 'Inter' and color to red."*
* *"Generate a storyboard for a cinematic sci-fi intro."*
* *"Add a TTS layer to my second scene that says 'Welcome to the future' using the `en-US-Journey-F` voice."*
* *"Create a React data-viz chart and embed it using a web scene layer."*
* *"Render a preview of my current project and give me the link."*

### Supported Tools

When you run `vidra mcp`, it exposed 15 tools for AI agents:

1.  `vidra.create_project` - Scaffolds a complete project (VidraScript + config).
2.  `vidra.add_scene` - Injects a new scene block into an existing timeline.
3.  `vidra.edit_layer` - Edits specific properties (text, size, color) using a semantic path (e.g., `project.scenes[0].layers[title].content.text`).
4.  `vidra.set_style` - Modifies colors, typography, or spacing globally or per-layer.
5.  `vidra.apply_brand_kit` - Installs a Brand Kit and maps its variables (`@brand.primary`) across the project.
6.  `vidra.add_asset` - Registers images, video, or audio to the project's AssetRegistry.
7.  `vidra.render_preview` - Queues a fast, low-res cloud render to visualize edits.
8.  `vidra.storyboard` - Takes a text prompt ("A sci-fi title intro") and converts it into a populated scene grid.
9.  `vidra.list_templates` - Browse built-in starter kits.
10. `vidra.add_resource` - Download and install components from Vidra Commons.
11. `vidra.list_resources` - Search Vidra Commons.
12. `vidra.share` - Generates a public URL for the current project state.
13. `vidra.generate_web_code` - Save HTML/React files to `web/` for embedding.
14. `vidra.add_web_scene` - Add a web layer to a given scene.
15. `vidra.edit_web_scene` - Edit viewport, source, variables, etc of a web layer.

## Conversational Editing & Copilots

By giving an AI agent access to the `vidra.edit_layer` tool, you achieve true conversational video editing:

**User:** "Make the title bigger and change the background to dark blue."

The AI doesn't need to rewrite the entire script. It reads the IR graph (using `vidra.inspect` optionally), parses the paths, and issues:

*   `vidra.edit_layer(path: "/scenes/0/layers/title/content/size", value: "120")`
*   `vidra.edit_layer(path: "/scenes/0/layers/bg/content/color", value: "#00008b")`

## Native AI Primitives

Aside from the MCP server *controlling* Vidra, Vidra incorporates AI *primitives* into the language itself.

### Text-to-Speech (TTS)

```javascript
layer("narration") {
    tts("Welcome to our new product launch.", voice: "en-US-Journey-F", volume: 1.0)
}
```

This compiles to `LayerContentNode::TTS`. Vidra Cloud (or local configured providers) orchestrates the audio generation, caching, and mapping the generated duration directly to the layer's lifecycle.

### Auto Captioning

```javascript
layer("captions") {
    autocaption("assets/voiceover.mp3", font: "Inter", size: 48, color: #ffffff)
}
```

Vidra sends the audio to a Whisper (or similar) model, receives word-level timestamps, and dynamically compiles an animation graph triggering `opacity` / `scale` keyframes on each word perfectly synchronized with the audio.

## Local AI Providers (Phase 4)

Vidra can materialize AI primitives (like `tts(...)`) into real cached assets during `vidra render`.

### Enable AI materialization

In your project folder, set `ai.enabled = true` in `vidra.config.toml`:

```toml
[ai]
enabled = true

[ai.openai]
# OpenAI-compatible base URL (can be OpenAI, Groq OpenAI-compatible, etc.)
base_url = "https://api.openai.com"
api_key_env = "OPENAI_API_KEY"
tts_model = "gpt-4o-mini-tts"
tts_format = "mp3"
transcribe_model = "whisper-1"

[ai.gemini]
# Optional: Gemini provider (HTTP). Used today for caption text refinement after transcription.
base_url = "https://generativelanguage.googleapis.com"
api_key_env = "GEMINI_API_KEY"
model = "gemini-1.5-flash"
caption_refine = false

[ai.removebg]
base_url = "https://api.remove.bg"
api_key_env = "REMOVEBG_API_KEY"
```

Then export the key in your shell:

```bash
export OPENAI_API_KEY="..."

# Optional, only if ai.gemini.caption_refine = true
export GEMINI_API_KEY="..."
```

For ElevenLabs TTS, set:

```bash
export ELEVENLABS_API_KEY="..."
```

And use an ElevenLabs voice id by prefixing the `tts` voice with `eleven:`:

```javascript
layer("narration") {
  tts("Hello from ElevenLabs", "eleven:YOUR_VOICE_ID")
}
```

Rendered TTS audio is cached under `resources.cache_dir` (or `ai.cache_dir` if set) so repeated renders and `--data` batch runs reuse the same output.

### AutoCaptions

When your script contains:

```javascript
layer("captions") {
  autocaption("assets/voiceover.mp3", font: "Inter", size: 48, color: #ffffff)
}
```

`vidra render` will:
- Transcribe the audio via the OpenAI-compatible transcription endpoint.
- Cache the `verbose_json` response in the AI cache.
- Replace the `AutoCaption` node with timed text child layers (segment-based) so captions render in the normal pipeline.

### Background removal (remove.bg)

Use it as an effect on an `image(...)` layer:

```javascript
layer("person") {
  image("assets/person.png")
  effect(removeBackground)
}
```

When enabled, `vidra render` calls remove.bg, caches the returned PNG-with-alpha, and swaps the layer’s image asset to the cached cutout.

## WASM / JS bridge (web / React Native)

In web/mobile runtimes, Vidra expects the host (JS) to perform network calls (provider APIs) and caching, then feed the results into the WASM engine as IR patches.

- Cache keys: use the same deterministic SHA-256 key strings as the Rust CLI so outputs are stable across platforms.
- Storage: use IndexedDB (recommended) or another persistent cache.
- Patch flow:
  - AutoCaptions: JS transcribes audio → calls `materialize_autocaption_layer(irJson, layerId, segmentsJson)`.
  - Background removal: JS calls provider → `load_image_asset(newAssetId, pngBytes)` → `apply_remove_background_patch(irJson, layerId, newAssetId)`.

The `@sansavision/vidra-player` package exposes cache-key helpers in `src/ai.ts` that mirror the Rust CLI’s key derivations.

## Future: Generative Video and Image Assets

Vidra is asset-agnostic. In the future, rather than `image("assets/bg.png")`, you can write:

```javascript
layer("background") {
    generate_image("A cinematic futuristic cityscape at sunset", seed: 42)
}
```

Vidra will treat generation models simply as just-in-time assets, piping the output directly into the GPU compositor.
