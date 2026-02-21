import type { Project, Scene, Layer, LayerContent, Animation, AnimatableProperty, Easing, AssetType, Color } from "./types.js";
export type * from "./types.js";
export declare class ProjectBuilder {
    private project;
    constructor(width: number, height: number, fps: number);
    background(color: Color): this;
    addAsset(asset_type: AssetType, id: string, path: string, name?: string): this;
    addScene(scene: Scene): this;
    build(): Project;
}
export declare class SceneBuilder {
    private scene;
    constructor(id: string, duration: number);
    addLayer(layer: Layer): this;
    build(): Scene;
}
export declare class LayerBuilder {
    private layer;
    constructor(id: string, content: LayerContent);
    position(x: number, y: number): this;
    scale(filter: number): this;
    scaleXY(x: number, y: number): this;
    rotation(angle: number): this;
    opacity(val: number): this;
    addChild(child: Layer): this;
    addAnimation(animation: Animation): this;
    build(): Layer;
}
export declare class AnimationBuilder {
    private animation;
    constructor(property: AnimatableProperty);
    delay(delaySeconds: number): this;
    addKeyframe(timeSeconds: number, value: number, easing: Easing): this;
    build(): Animation;
}
export declare const ColorUtils: {
    rgba(r: number, g: number, b: number, a?: number): Color;
    hex(hexCode: string): Color;
};
//# sourceMappingURL=index.d.ts.map