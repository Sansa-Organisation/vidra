/**
 * Generates a random UUID (v4).
 */
function uuidv4() {
    return "10000000-1000-4000-8000-100000000000".replace(/[018]/g, (c) => (+c ^
        (Math.floor(Math.random() * 256) & (15 >> (+c / 4)))).toString(16));
}
export class ProjectBuilder {
    project;
    constructor(width, height, fps) {
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
    background(color) {
        this.project.settings.background = color;
        return this;
    }
    addAsset(asset_type, id, path, name) {
        this.project.assets.assets[id] = {
            id,
            asset_type,
            path,
            name: name ?? null,
        };
        return this;
    }
    addScene(scene) {
        this.project.scenes.push(scene);
        return this;
    }
    build() {
        return this.project;
    }
}
export class SceneBuilder {
    scene;
    constructor(id, duration) {
        this.scene = {
            id,
            duration: { seconds: duration },
            layers: [],
        };
    }
    addLayer(layer) {
        this.scene.layers.push(layer);
        return this;
    }
    build() {
        return this.scene;
    }
}
export class LayerBuilder {
    layer;
    constructor(id, content) {
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
    position(x, y) {
        this.layer.transform.position = { x, y };
        return this;
    }
    scale(filter) {
        this.layer.transform.scale = { x: filter, y: filter };
        return this;
    }
    scaleXY(x, y) {
        this.layer.transform.scale = { x, y };
        return this;
    }
    rotation(angle) {
        this.layer.transform.rotation = angle;
        return this;
    }
    opacity(val) {
        this.layer.transform.opacity = val;
        return this;
    }
    addChild(child) {
        this.layer.children.push(child);
        return this;
    }
    addAnimation(animation) {
        this.layer.animations.push(animation);
        return this;
    }
    build() {
        return this.layer;
    }
}
export class AnimationBuilder {
    animation;
    constructor(property) {
        this.animation = {
            property,
            keyframes: [],
            delay: { seconds: 0 },
        };
    }
    delay(delaySeconds) {
        this.animation.delay = { seconds: delaySeconds };
        return this;
    }
    addKeyframe(timeSeconds, value, easing) {
        this.animation.keyframes.push({
            time: { seconds: timeSeconds },
            value,
            easing,
        });
        return this;
    }
    build() {
        return this.animation;
    }
}
export const ColorUtils = {
    rgba(r, g, b, a = 255) {
        return { r, g, b, a };
    },
    hex(hexCode) {
        const clean = hexCode.replace("#", "");
        if (clean.length === 6) {
            return {
                r: parseInt(clean.substring(0, 2), 16),
                g: parseInt(clean.substring(2, 4), 16),
                b: parseInt(clean.substring(4, 6), 16),
                a: 255,
            };
        }
        else if (clean.length === 8) {
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
//# sourceMappingURL=index.js.map