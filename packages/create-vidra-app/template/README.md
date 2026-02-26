# {{PROJECT_NAME}}

A video project powered by [Vidra](https://github.com/Sansa-Organisation/vidra).

## Quick Start

```bash
npm install
npm run dev          # Open browser preview
npm run build:video  # Generate project IR JSON
```

## Render to MP4

```bash
# Using the Vidra CLI:
vidra render video.vidra -o output.mp4

# Or use the visual editor:
vidra editor video.vidra
```

## Project Structure

```
├── index.html        ← Browser preview (Vite dev server)
├── src/
│   └── video.js      ← SDK-based video builder
├── web/
│   └── chart.html    ← Web scene with capture bridge
├── video.vidra       ← VidraScript DSL version
└── package.json
```

## Packages Used

| Package | Purpose |
|---------|---------|
| `@sansavision/vidra-sdk` | Build video projects programmatically |
| `@sansavision/vidra-player` | WASM-powered browser renderer |
| `@sansavision/vidra-web-capture` | Bridge for web scenes |

## Learn More

- [Vidra Documentation](https://github.com/Sansa-Organisation/vidra/tree/main/docs)
- [VidraScript Reference](https://github.com/Sansa-Organisation/vidra/blob/main/docs/vidrascript.md)
- [Web Scenes Guide](https://github.com/Sansa-Organisation/vidra/blob/main/docs/web-scenes.md)
