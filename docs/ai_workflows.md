# AI & the Model Context Protocol (MCP)

Vidra is designed to be fully programmable, which naturally extends to being entirely promptable by Large Language Models. 

Instead of an AI trying to write a complex, custom Adobe After Effects script or stitching obscure Python image libraries, Vidra provides an **MCP Server** out of the box.

## The Vidra MCP Server

The **Model Context Protocol (MCP)** gives any AI assistant (like Claude Desktop, cursor, or independent agents) a standardized way to read and write context. Vidra implements a full MCP server mapped directly to its internal engine API.

```bash
vidra mcp
# or without installation: bunx @sansavision/vidra mcp
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
        "@sansavision/vidra",
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
* *"Render a preview of my current project and give me the link."*

### Supported Tools

When you run `vidra mcp`, it exposed 12 tools for AI agents:

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

## Future: Generative Video and Image Assets

Vidra is asset-agnostic. In the future, rather than `image("assets/bg.png")`, you can write:

```javascript
layer("background") {
    generate_image("A cinematic futuristic cityscape at sunset", seed: 42)
}
```

Vidra will treat generation models simply as just-in-time assets, piping the output directly into the GPU compositor.
