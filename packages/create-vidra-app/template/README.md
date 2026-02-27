# {{PROJECT_NAME}}

A video project powered by [Vidra](https://github.com/Sansa-Organisation/vidra).

## Quick Start

```bash
npm install
npm run dev          # Open browser — shows SDK output preview + live web scene
npm run build:video  # Build the SDK project and output JSON IR to stdout
```

## Render to MP4

Use the Vidra CLI with the VidraScript file:
```bash
npx @sansavision/vidra render video.vidra -o output.mp4
```

Or open the visual editor:
```bash
npx @sansavision/vidra editor video.vidra
```

## Project Structure

```
├── index.html        ← Dev page: SDK output preview + embedded web scene
├── src/
│   └── video.js      ← SDK-based video builder (outputs JSON IR)
├── web/
│   └── chart.html    ← Web scene using @sansavision/vidra-web-capture
├── video.vidra       ← VidraScript DSL (render with CLI)
└── package.json
```

## Packages Used

| Package | Where | Purpose |
|---------|-------|---------|
| `@sansavision/vidra-sdk` | `src/video.js`, `index.html` | Build video projects programmatically |
| `@sansavision/vidra-web-capture` | `web/chart.html` | Bridge for web scenes to sync with video timeline |

## Learn More

- [Vidra Documentation](https://github.com/Sansa-Organisation/vidra/tree/main/docs)
- [Web Scenes Guide](https://github.com/Sansa-Organisation/vidra/blob/main/docs/web-scenes.md)
