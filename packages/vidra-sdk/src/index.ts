// ─── Vidra SDK — Programmatic Video Builder ────────────────────────
// Build Vidra videos using TypeScript instead of VidraScript DSL.
//
// Usage:
//   import { Project, Scene, Layer, Solid, Text, Easing } from "@sansavision/vidra-sdk";
//
//   const project = new Project(1920, 1080, 60);
//   const scene = new Scene("intro", 3.0);
//   scene.addLayer(new Layer("bg").solid("#1a1a2e"));
//   scene.addLayer(
//       new Layer("title")
//           .text("Hello!", "Inter", 100, "#ffffff")
//           .position(960, 540)
//           .animate("opacity", 0, 1, 1.0, "easeOut")
//   );
//   project.addScene(scene);
//   await project.render("output.mp4");

import type {
    ProjectIR,
    SceneIR,
    LayerIR,
    LayerContent,
    Animation,
    AnimatableProperty,
    Easing as EasingType,
    Color,
    AssetType,
    LayerEffect,
} from "./types.js";

export type * from "./types.js";

// ─── Color Utilities ────────────────────────────────────────────────

export function rgba(r: number, g: number, b: number, a: number = 255): Color {
    return { r, g, b, a };
}

export function hex(hexCode: string): Color {
    const clean = hexCode.replace("#", "");
    if (clean.length === 6) {
        return {
            r: parseInt(clean.substring(0, 2), 16),
            g: parseInt(clean.substring(2, 4), 16),
            b: parseInt(clean.substring(4, 6), 16),
            a: 255,
        };
    } else if (clean.length === 8) {
        return {
            r: parseInt(clean.substring(0, 2), 16),
            g: parseInt(clean.substring(2, 4), 16),
            b: parseInt(clean.substring(4, 6), 16),
            a: parseInt(clean.substring(6, 8), 16),
        };
    }
    return { r: 0, g: 0, b: 0, a: 255 };
}

function colorToHex(c: Color): string {
    const r = c.r.toString(16).padStart(2, "0");
    const g = c.g.toString(16).padStart(2, "0");
    const b = c.b.toString(16).padStart(2, "0");
    return `#${r}${g}${b}`;
}

// ─── UUID ───────────────────────────────────────────────────────────

function uuidv4(): string {
    return "10000000-1000-4000-8000-100000000000".replace(/[018]/g, (c) =>
        (
            +c ^
            (Math.floor(Math.random() * 256) & (15 >> (+c / 4)))
        ).toString(16)
    );
}

// ─── Easing Constants ───────────────────────────────────────────────

export const Easing = {
    Linear: "Linear" as EasingType,
    EaseIn: "EaseIn" as EasingType,
    EaseOut: "EaseOut" as EasingType,
    EaseInOut: "EaseInOut" as EasingType,
    CubicIn: "CubicIn" as EasingType,
    CubicOut: "CubicOut" as EasingType,
    CubicInOut: "CubicInOut" as EasingType,
    Step: "Step" as EasingType,
} as const;

// ─── Layer Builder ──────────────────────────────────────────────────

export class Layer {
    private _layer: LayerIR;

    constructor(id: string) {
        this._layer = {
            id,
            content: "Empty",
            transform: {
                position: { x: 0, y: 0 },
                scale: { x: 1, y: 1 },
                rotation: 0,
                opacity: 1,
                anchor: { x: 0.5, y: 0.5 },
            },
            blend_mode: "Normal",
            animations: [],
            effects: [],
            visible: true,
            children: [],
        };
    }

    // ── Content setters ───────────────────────────────────────────

    text(content: string, fontFamily: string = "Inter", fontSize: number = 48, color: string | Color = "#ffffff"): this {
        const c = typeof color === "string" ? hex(color) : color;
        this._layer.content = { Text: { text: content, font_family: fontFamily, font_size: fontSize, color: c } };
        return this;
    }

    solid(color: string | Color): this {
        const c = typeof color === "string" ? hex(color) : color;
        this._layer.content = { Solid: { color: c } };
        return this;
    }

    image(assetId: string): this {
        this._layer.content = { Image: { asset_id: assetId } };
        return this;
    }

    video(assetId: string, trimStart: number = 0, trimEnd?: number): this {
        this._layer.content = {
            Video: {
                asset_id: assetId,
                trim_start: { seconds: trimStart },
                trim_end: trimEnd !== undefined ? { seconds: trimEnd } : null,
            },
        };
        return this;
    }

    audio(assetId: string, volume: number = 1.0, trimStart: number = 0, trimEnd?: number): this {
        this._layer.content = {
            Audio: {
                asset_id: assetId,
                trim_start: { seconds: trimStart },
                trim_end: trimEnd !== undefined ? { seconds: trimEnd } : null,
                volume,
            },
        };
        return this;
    }

    shape(type: "rect", opts: { width: number; height: number; radius?: number; fill?: string | Color }): this;
    shape(type: "circle", opts: { radius: number; fill?: string | Color }): this;
    shape(type: "ellipse", opts: { rx: number; ry: number; fill?: string | Color }): this;
    shape(type: string, opts: Record<string, unknown>): this {
        const fill = opts.fill ? (typeof opts.fill === "string" ? hex(opts.fill as string) : opts.fill as Color) : null;
        if (type === "rect") {
            this._layer.content = {
                Shape: { shape: { Rect: { width: opts.width as number, height: opts.height as number, radius: (opts.radius as number) ?? 0 } }, fill, stroke: null, stroke_width: 0 },
            };
        } else if (type === "circle") {
            this._layer.content = {
                Shape: { shape: { Circle: { radius: opts.radius as number } }, fill, stroke: null, stroke_width: 0 },
            };
        } else if (type === "ellipse") {
            this._layer.content = {
                Shape: { shape: { Ellipse: { rx: opts.rx as number, ry: opts.ry as number } }, fill, stroke: null, stroke_width: 0 },
            };
        }
        return this;
    }

    tts(content: string, voice: string = "default", volume: number = 1.0): this {
        this._layer.content = { TTS: { text: content, voice, volume } };
        return this;
    }

    // ── Transform setters ─────────────────────────────────────────

    position(x: number, y: number): this {
        this._layer.transform.position = { x, y };
        return this;
    }

    scale(factor: number): this;
    scale(x: number, y: number): this;
    scale(x: number, y?: number): this {
        this._layer.transform.scale = { x, y: y ?? x };
        return this;
    }

    rotation(degrees: number): this {
        this._layer.transform.rotation = degrees;
        return this;
    }

    opacity(value: number): this {
        this._layer.transform.opacity = value;
        return this;
    }

    anchor(x: number, y: number): this {
        this._layer.transform.anchor = { x, y };
        return this;
    }

    // ── Animation ─────────────────────────────────────────────────

    animate(
        property: AnimatableProperty | "positionX" | "positionY" | "scaleX" | "scaleY" | "rotation" | "opacity",
        from: number,
        to: number,
        durationSec: number,
        easing: EasingType = "Linear",
        delaySec: number = 0
    ): this {
        const propMap: Record<string, AnimatableProperty> = {
            positionX: "PositionX", positionY: "PositionY",
            scaleX: "ScaleX", scaleY: "ScaleY",
            rotation: "Rotation", opacity: "Opacity",
            PositionX: "PositionX", PositionY: "PositionY",
            ScaleX: "ScaleX", ScaleY: "ScaleY",
            Rotation: "Rotation", Opacity: "Opacity",
        };
        const mapped = propMap[property] ?? property as AnimatableProperty;

        this._layer.animations.push({
            property: mapped,
            keyframes: [
                { time: { seconds: 0 }, value: from, easing },
                { time: { seconds: durationSec }, value: to, easing },
            ],
            delay: { seconds: delaySec },
        });
        return this;
    }

    // ── Effects ────────────────────────────────────────────────────

    blur(radius: number): this {
        this._layer.effects.push({ Blur: { radius } });
        return this;
    }

    dropShadow(offsetX: number, offsetY: number, blur: number, color: string | Color = "#000000"): this {
        const c = typeof color === "string" ? hex(color) : color;
        this._layer.effects.push({ DropShadow: { offset_x: offsetX, offset_y: offsetY, blur, color: c } });
        return this;
    }

    // ── Children ──────────────────────────────────────────────────

    addChild(child: Layer): this {
        this._layer.children.push(child.build());
        return this;
    }

    // ── Build ─────────────────────────────────────────────────────

    build(): LayerIR {
        return this._layer;
    }
}

// ─── Scene Builder ──────────────────────────────────────────────────

export class Scene {
    private _scene: SceneIR;

    constructor(id: string, durationSec: number) {
        this._scene = {
            id,
            duration: { seconds: durationSec },
            layers: [],
        };
    }

    addLayer(layer: Layer): this {
        this._scene.layers.push(layer.build());
        return this;
    }

    addLayers(...layers: Layer[]): this {
        for (const l of layers) {
            this._scene.layers.push(l.build());
        }
        return this;
    }

    build(): SceneIR {
        return this._scene;
    }
}

// ─── Project Builder ────────────────────────────────────────────────

export class Project {
    private _project: ProjectIR;

    constructor(width: number, height: number, fps: number) {
        this._project = {
            id: uuidv4(),
            settings: { width, height, fps, background: { r: 0, g: 0, b: 0, a: 255 } },
            assets: { assets: {} },
            scenes: [],
        };
    }

    background(color: string | Color): this {
        this._project.settings.background = typeof color === "string" ? hex(color) : color;
        return this;
    }

    addAsset(type: AssetType, id: string, path: string, name?: string): this {
        this._project.assets.assets[id] = { id, asset_type: type, path, name: name ?? null };
        return this;
    }

    addScene(scene: Scene): this {
        this._project.scenes.push(scene.build());
        return this;
    }

    // ── Output: IR JSON ───────────────────────────────────────────

    toJSON(): ProjectIR {
        return this._project;
    }

    toJSONString(pretty: boolean = true): string {
        return JSON.stringify(this._project, null, pretty ? 2 : undefined);
    }

    // ── Output: VidraScript DSL ───────────────────────────────────

    toVidraScript(): string {
        const p = this._project;
        const lines: string[] = [];
        lines.push(`project(${p.settings.width}, ${p.settings.height}, ${p.settings.fps}) {`);

        for (const scene of p.scenes) {
            lines.push(`    scene("${scene.id}", ${scene.duration.seconds}s) {`);
            for (const layer of scene.layers) {
                this._emitLayer(lines, layer, 2);
            }
            lines.push(`    }`);
        }

        lines.push(`}`);
        return lines.join("\n") + "\n";
    }

    private _emitLayer(lines: string[], layer: LayerIR, indent: number): void {
        const pad = "    ".repeat(indent);
        lines.push(`${pad}layer("${layer.id}") {`);

        // Content
        const content = layer.content;
        if (content === "Empty") {
            // no content line
        } else if ("Solid" in content) {
            lines.push(`${pad}    solid(${colorToHex(content.Solid.color)})`);
        } else if ("Text" in content) {
            const t = content.Text;
            lines.push(`${pad}    text("${t.text}", font: "${t.font_family}", size: ${t.font_size}, color: ${colorToHex(t.color)})`);
        } else if ("Image" in content) {
            lines.push(`${pad}    image("${content.Image.asset_id}")`);
        } else if ("Video" in content) {
            lines.push(`${pad}    video("${content.Video.asset_id}")`);
        } else if ("Audio" in content) {
            lines.push(`${pad}    audio("${content.Audio.asset_id}", volume: ${content.Audio.volume})`);
        } else if ("TTS" in content) {
            lines.push(`${pad}    tts("${content.TTS.text}", "${content.TTS.voice}")`);
        }

        // Position
        const pos = layer.transform.position;
        if (pos.x !== 0 || pos.y !== 0) {
            lines.push(`${pad}    position(${pos.x}, ${pos.y})`);
        }

        // Animations
        for (const anim of layer.animations) {
            if (anim.keyframes.length >= 2) {
                const from = anim.keyframes[0]!;
                const to = anim.keyframes[anim.keyframes.length - 1]!;
                const easingStr = from.easing.charAt(0).toLowerCase() + from.easing.slice(1);
                const propMap: Record<string, string> = {
                    PositionX: "positionX", PositionY: "positionY",
                    ScaleX: "scaleX", ScaleY: "scaleY",
                    Rotation: "rotation", Opacity: "opacity",
                };
                const prop = propMap[anim.property] ?? anim.property;
                lines.push(`${pad}    animation(${prop}, from: ${from.value}, to: ${to.value}, duration: ${to.time.seconds}s, easing: ${easingStr})`);
            }
        }

        // Children
        for (const child of layer.children) {
            this._emitLayer(lines, child, indent + 1);
        }

        lines.push(`${pad}}`);
    }

    // ── Output: Render via CLI ────────────────────────────────────

    async render(outputPath: string, options?: { cliPath?: string }): Promise<string> {
        const fs = await import("node:fs");
        const { execSync } = await import("node:child_process");
        const path = await import("node:path");
        const os = await import("node:os");

        // Write the VidraScript to a temp file
        const tmpDir = os.tmpdir();
        const tmpFile = path.join(tmpDir, `vidra_sdk_${Date.now()}.vidra`);
        fs.writeFileSync(tmpFile, this.toVidraScript());

        const cli = options?.cliPath ?? "vidra";

        try {
            const result = execSync(`${cli} render "${tmpFile}" --output "${outputPath}"`, {
                encoding: "utf-8",
                stdio: ["pipe", "pipe", "pipe"],
            });
            return result;
        } finally {
            // Clean up temp file
            try { fs.unlinkSync(tmpFile); } catch { /* ignore */ }
        }
    }
}
