// ─── Vidra SDK — Core Types ─────────────────────────────────────────
// These types are a 1:1 mirror of the Rust `vidra-ir` crate output.

export interface Color {
    r: number;
    g: number;
    b: number;
    a: number;
}

export interface Point2D {
    x: number;
    y: number;
}

export interface Transform2D {
    position: Point2D;
    scale: Point2D;
    rotation: number;
    opacity: number;
    anchor: Point2D;
}

export type BlendMode =
    | "Normal"
    | "Multiply"
    | "Screen"
    | "Overlay"
    | "Darken"
    | "Lighten"
    | "Add";

export type AssetType = "Image" | "Video" | "Audio" | "Font";

export type AssetId = string;

export interface Asset {
    id: AssetId;
    asset_type: AssetType;
    path: string;
    name: string | null;
}

export interface AssetRegistry {
    assets: Record<AssetId, Asset>;
}

export type LayerId = string;

export type ShapeType =
    | { Rect: { width: number; height: number; radius: number } }
    | { Circle: { radius: number } }
    | { Ellipse: { rx: number; ry: number } };

export interface Duration {
    seconds: number;
}

export type WebCaptureMode = "FrameAccurate" | "Realtime";

export type LayerContent =
    | { Text: { text: string; font_family: string; font_size: number; color: Color } }
    | { Image: { asset_id: AssetId } }
    | { Video: { asset_id: AssetId; trim_start: Duration; trim_end: Duration | null } }
    | { Audio: { asset_id: AssetId; trim_start: Duration; trim_end: Duration | null; volume: number } }
    | { Shape: { shape: ShapeType; fill: Color | null; stroke: Color | null; stroke_width: number } }
    | { Solid: { color: Color } }
    | { TTS: { text: string; voice: string; volume: number } }
    | { AutoCaption: { asset_id: AssetId; font_family: string; font_size: number; color: Color } }
    | { Web: { source: string; viewport_width: number; viewport_height: number; mode: WebCaptureMode; wait_for: string | null; variables: Record<string, number> } }
    | "Empty";

export type AnimatableProperty =
    | "PositionX"
    | "PositionY"
    | "ScaleX"
    | "ScaleY"
    | "Rotation"
    | "Opacity"
    | "FontSize"
    | "ColorR"
    | "ColorG"
    | "ColorB"
    | "ColorA"
    | "CornerRadius"
    | "StrokeWidth"
    | "CropTop"
    | "CropRight"
    | "CropBottom"
    | "CropLeft"
    | "Volume"
    | "BlurRadius"
    | "BrightnessLevel";

export type Easing =
    | "Linear"
    | "EaseIn"
    | "EaseOut"
    | "EaseInOut"
    | "CubicIn"
    | "CubicOut"
    | "CubicInOut"
    | "Step";

export interface Keyframe {
    time: Duration;
    value: number;
    easing: Easing;
}

export interface Animation {
    property: AnimatableProperty;
    keyframes: Keyframe[];
    delay: Duration;
}

export type LayerEffect =
    | { Blur: { radius: number } }
    | { DropShadow: { offset_x: number; offset_y: number; blur: number; color: Color } };

export interface LayerIR {
    id: LayerId;
    content: LayerContent;
    transform: Transform2D;
    blend_mode: BlendMode;
    animations: Animation[];
    effects: LayerEffect[];
    visible: boolean;
    children: LayerIR[];
}

export interface SceneIR {
    id: string;
    duration: Duration;
    layers: LayerIR[];
}

export interface ProjectSettings {
    width: number;
    height: number;
    fps: number;
    background: Color;
}

export interface ProjectIR {
    id: string;
    settings: ProjectSettings;
    assets: AssetRegistry;
    scenes: SceneIR[];
}
