import type {
    Project,
    ProjectSettings,
    Scene,
    Layer,
    LayerContent,
    Animation,
    AnimatableProperty,
    Easing,
    AssetType,
    Color,
} from "./types.js";

export type * from "./types.js";

/**
 * Generates a random UUID (v4).
 */
function uuidv4(): string {
    return "10000000-1000-4000-8000-100000000000".replace(/[018]/g, (c) =>
        (
            +c ^
            (Math.floor(Math.random() * 256) & (15 >> (+c / 4)))
        ).toString(16)
    );
}

export class ProjectBuilder {
    private project: Project;

    constructor(width: number, height: number, fps: number) {
        this.project = {
            id: uuidv4(),
            settings: {
                width,
                height,
                fps,
                background: { r: 0, g: 0, b: 0, a: 255 },
            },
            assets: { assets: {} },
            scenes: [],
        };
    }

    public background(color: Color): this {
        this.project.settings.background = color;
        return this;
    }

    public addAsset(
        asset_type: AssetType,
        id: string,
        path: string,
        name?: string
    ): this {
        this.project.assets.assets[id] = {
            id,
            asset_type,
            path,
            name: name ?? null,
        };
        return this;
    }

    public addScene(scene: Scene): this {
        this.project.scenes.push(scene);
        return this;
    }

    public build(): Project {
        return this.project;
    }
}

export class SceneBuilder {
    private scene: Scene;

    constructor(id: string, duration: number) {
        this.scene = {
            id,
            duration: { seconds: duration },
            layers: [],
        };
    }

    public addLayer(layer: Layer): this {
        this.scene.layers.push(layer);
        return this;
    }

    public build(): Scene {
        return this.scene;
    }
}

export class LayerBuilder {
    private layer: Layer;

    constructor(id: string, content: LayerContent) {
        this.layer = {
            id,
            content,
            transform: {
                position: { x: 0, y: 0 },
                scale: { x: 1, y: 1 },
                rotation: 0,
                opacity: 1,
                anchor: { x: 0, y: 0 },
            },
            blend_mode: "Normal",
            animations: [],
            visible: true,
            children: [],
        };
    }

    public position(x: number, y: number): this {
        this.layer.transform.position = { x, y };
        return this;
    }

    public scale(filter: number): this {
        this.layer.transform.scale = { x: filter, y: filter };
        return this;
    }

    public scaleXY(x: number, y: number): this {
        this.layer.transform.scale = { x, y };
        return this;
    }

    public rotation(angle: number): this {
        this.layer.transform.rotation = angle;
        return this;
    }

    public opacity(val: number): this {
        this.layer.transform.opacity = val;
        return this;
    }

    public addChild(child: Layer): this {
        this.layer.children.push(child);
        return this;
    }

    public addAnimation(animation: Animation): this {
        this.layer.animations.push(animation);
        return this;
    }

    public build(): Layer {
        return this.layer;
    }
}

export class AnimationBuilder {
    private animation: Animation;

    constructor(property: AnimatableProperty) {
        this.animation = {
            property,
            keyframes: [],
            delay: { seconds: 0 },
        };
    }

    public delay(delaySeconds: number): this {
        this.animation.delay = { seconds: delaySeconds };
        return this;
    }

    public addKeyframe(timeSeconds: number, value: number, easing: Easing): this {
        this.animation.keyframes.push({
            time: { seconds: timeSeconds },
            value,
            easing,
        });
        return this;
    }

    public build(): Animation {
        return this.animation;
    }
}

export const ColorUtils = {
    rgba(r: number, g: number, b: number, a: number = 255): Color {
        return { r, g, b, a };
    },
    hex(hexCode: string): Color {
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
    },
};
