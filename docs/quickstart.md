# Vidra Quickstart Guide

Welcome to Vidra! This guide will take you from installation to your first rendered video in under 5 minutes.

## 1. Installation

Install the Vidra CLI using our setup script:

```bash
curl -fsSL https://vidra.dev/install.sh | sh
```

Or build from source using Cargo:

```bash
cargo install --path crates/vidra-cli
```

Verify the installation:

```bash
vidra info
```

## 2. Create Your First Project

Let's create a new project called `my-first-video`:

```bash
vidra init my-first-video
cd my-first-video
```

This creates a new folder with a basic project structure, including a `main.vidra` file and a `vidra.config.toml`.

## 3. Write Some VidraScript

Open `main.vidra` in your favorite text editor. Replace its contents with this simple scene:

```javascript
project(1920, 1080, 60) {
    scene("intro", 4s) {
        // A solid background color
        layer("bg") {
            solid(#1a1a2e)
        }

        // A text layer with an animation
        layer("title") {
            text("Hello, Vidra!", font: "Inter", size: 80, color: #e94560)
            position(960, 540)
            
            // Fade in over the first second
            animation(opacity, from: 0, to: 1, duration: 1s, easing: ease-out)
            
            // Slide up slightly
            animation(position_y, from: 560, to: 540, duration: 1s, easing: ease-out)
        }
    }
}
```

## 4. Live Preview

You can see your changes in real-time by starting the development server:

```bash
vidra dev main.vidra
```

This opens a local web viewer at `http://localhost:8080` where you can preview your project as you code. Every time you save `main.vidra`, the preview will automatically update.

## 5. Render to Video

When you're happy with your video, you can render it to an actual MP4 file using the GPU:

```bash
vidra render main.vidra --output output.mp4
```

And that's it! You've successfully rendered your first video entirely from code.

## Next Steps

Now that you know the basics, here's what you can do next:
- **Learn the Language**: Read the [VidraScript Reference](vidrascript.md) to learn about components, brand kits, conditional logic, and AI layers.
- **Add Audio**: Learn how to compose audio and use automated TTS (`tts("Text", "Voice")`).
- **Use the AI Copilot**: Start the MCP server (`vidra mcp`) to integrate Vidra with your favorite AI agent, or use `vidra storyboard "A cinematic intro"` to generate scenes from text.
