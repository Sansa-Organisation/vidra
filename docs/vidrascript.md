# VidraScript Language Reference

VidraScript is a purpose-built domain-specific language (DSL) for declarative video composition. It offers structured syntax combining the declarative readability of CSS with the composability of modern UI frameworks (like React or SwiftUI).

## Project Structure

A `project` is the root node. It takes three parameters: `width`, `height`, and `fps` (Frames Per Second).

```javascript
project(1920, 1080, 60) {
    // scenes and components go here
}
```

## Scenes

A `scene` is a time-bounded segment of the timeline. Scenes execute sequentially. It takes a name and a duration.

```javascript
scene("intro", 5s) {
    // layers go here
}
```

## Layers

A `layer` is the fundamental renderable unit. Layers are stacked bottom-to-top (the last layer in a block renders on top).

```javascript
layer("background") {
    solid(#1a1a2e)
}
```

Every layer contains exactly one **Layer Content**, followed by optional properties (like position, animation, and children).

### Content Types

| Content Type | Syntax | Description |
|---|---|---|
| Solid | `solid(#hex)` | A solid color background. |
| Text | `text("String", font: "Inter", size: 48, color: #ffffff)` | Render text with automatic font management. |
| Image | `image("path/to.png")` | Load a static image (.png, .jpeg). |
| Video | `video("path/to.mp4", trim_start: 0s, trim_end: 5s)` | Load and play a video clip. |
| Audio | `audio("path/to.mp3", volume: 1.0)` | Play audio. Cannot be transformed visually. |
| TTS | `tts("Text to speak", "en-US-Standard-A")` | AI text-to-speech. Uses cloud orchestration. |
| AutoCaption| `autocaption("path/to.mp3", font: "Inter", size: 32)` | Automatically transcribe and layout animated words. |
| Shape | `shape(rect, fill: #ff0000, width: 100, height: 100)` | Primitive shapes (`rect`, `circle`, etc). |
| Component | `use("Name", prop: "value")` | Place an instantiated component block. |

### Properties

Properties apply transformations and animations to layers.

*   `position(x, y)` sets the anchor position.
*   `scale(x, y)` adjusts size.
*   `animation(property, from: val, to: val, duration: time, easing: type)` animates the layer.
*   `effect(type, intensity)` applies a visual post-process effect (e.g., `blur`, `grayscale`, `invert`).

Example:

```javascript
layer("logo") {
    image("assets/logo.png")
    position(960, 540)
    scale(1.5, 1.5)
    animation(opacity, from: 0, to: 1, duration: 1s, easing: ease-out)
    animation(position_y, from: 600, to: 540, duration: 2s, easing: ease-out)
    effect(blur, 10.0)
}
```

## Components

Components are reusable blocks that encapsulate one or more layers, accepting props.

```javascript
component(TitleCard, text: String, color: Color) {
    layer("bg") {
        solid(color)
    }
    layer("text") {
        text(text, font: "Inter", size: 100, color: #ffffff)
        position(960, 540)
    }
}
```

To use a component:

```javascript
layer("scene1_title") {
    use("TitleCard", text: "Welcome!", color: #e94560)
}
```

### Slots

Components can accept children by declaring `slot()` in a layer.

```javascript
component(Container) {
    layer("wrapper") {
        solid(#000000)
        slot()
    }
}
```

Usage:

```javascript
use("Container") {
    layer("child1") { text("I am inside!") }
}
```

### Variants

Components can define variants to encapsulate specific styles or overrides.

```javascript
component(Button, label: String) {
    layer("base") {
        solid(#000000)
    }
    
    variant("primary") {
        layer("base") {
            solid(#e94560)
        }
    }
}

use("Button", label: "Click", variant: "primary")
```

## Brand Kits

References to brand assets can be passed directly into layout attributes using the `@brand` prefix. These are resolved at compile time from the workspace `vidra.config.toml` (or cloud metadata).

```javascript
layer("bg") {
    solid(@brand.colors.primary)
}

layer("heading") {
    text("Hello", font: @brand.fonts.heading, color: @brand.colors.text)
}
```

## Conditionals and Loops (Logic)

You can use basic logic nodes to toggle visibility or instance elements multiple times.

```javascript
if (show_disclaimer) {
    layer("disclaimer") {
        text("Terms and conditions apply.", size: 12)
        position(1920, 1060)
    }
} else {
    // ...
}
```

## Responsive Layouts (When / Aspect)

Vidra automatically handles multi-format rendering using `layout rules` and `when aspect()`.

```javascript
layout rules {
    when aspect(9:16) {
        layer("title") {
            position(540, 1500)
            scale(1.2, 1.2)
        }
    }
}
```

When rendering with `vidra render --targets 16:9,9:16`, the engine will compile the IR graph specifically overriding properties dynamically based on the target aspect ratio, ensuring perfect composition across formats inside the exact same project tree.
