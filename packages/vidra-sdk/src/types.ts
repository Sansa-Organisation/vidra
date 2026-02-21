// Core geometric and structural types generated out of Vidra IR
export type Color = {
    r: number;
    g: number;
    b: number;
    a: number;
};

export type Point2D = {
    x: number;
    y: number;
};

export type Transform2D = {
    position: Point2D;
    scale: Point2D;
    rotation: number;
    opacity: number;
    anchor: Point2D;
};

export type BlendMode =
    | "Normal"
    | "Multiply"
    | "Screen"
    | "Overlay"
    | "Darken"
    | "Lighten"
    | "Add";

// Asset system
export type AssetType = "Image" | "Video" | "Audio" | "Font";

export type AssetId = string;

export type Asset = {
    id: AssetId;
    asset_type: AssetType;
    path: string;
    name: string | null;
};

export type AssetRegistry = {
    assets: Record<AssetId, Asset>;
};

export type LayerId = string;

export type ShapeType =
    | { Rectangle: { width: number; height: number; radius: number } }
    | { Circle: { radius: number } }
    | { Ellipse: { radius_x: number; radius_y: number } };

export type Duration = {
    seconds: number;
};

export type LayerContent =
    | { Text: { text: string; font_family: string; font_size: number; color: Color } }
    | { Image: { asset_id: AssetId } }
    | { Video: { asset_id: AssetId; trim_start: Duration; trim_end: Duration | null } }
    | { Audio: { asset_id: AssetId; trim_start: Duration; trim_end: Duration | null; volume: number } }
    | { Shape: { shape: ShapeType; fill: Color | null; stroke: Color | null; stroke_width: number } }
    | { Solid: { color: Color } }
    | "Empty";

export type AnimatableProperty =
    | "PositionX"
    | "PositionY"
    | "ScaleX"
    | "ScaleY"
    | "Rotation"
    | "Opacity";

export type Easing = "Linear" | "EaseIn" | "EaseOut" | "EaseInOut" | "Step";

export type Keyframe = {
    time: Duration;
    value: number;
    easing: Easing;
};

export type Animation = {
    property: AnimatableProperty;
    keyframes: Keyframe[];
    delay: Duration;
};

export type Layer = {
    id: LayerId;
    content: LayerContent;
    transform: Transform2D;
    blend_mode: BlendMode;
    animations: Animation[];
    visible: boolean;
    children: Layer[];
};

export type SceneId = string;

export type Scene = {
    id: SceneId;
    duration: Duration;
    layers: Layer[];
};

export type ProjectSettings = {
    width: number;
    height: number;
    fps: number;
    background: Color;
};

export type Project = {
    id: string;
    settings: ProjectSettings;
    assets: AssetRegistry;
    scenes: Scene[];
};
