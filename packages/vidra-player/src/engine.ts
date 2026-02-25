// ─── Vidra Engine — WASM Loader & Render Loop ─────────────────────
// This module loads the WASM module, manages the <canvas>, and drives
// the requestAnimationFrame render loop.

import init, {
    parse_and_compile,
    render_frame,
    render_frame_from_source,
    get_project_info,
    load_image_asset,
    materialize_autocaption_layer,
    apply_remove_background_patch,
    get_web_layers_state,
    version,
} from "../wasm/vidra_wasm.js";

export interface CaptionSegment {
    start_s: number;
    end_s: number;
    text: string;
}

export interface ProjectInfo {
    width: number;
    height: number;
    fps: number;
    totalFrames: number;
    totalDuration: number;
    sceneCount: number;
}

export interface WebLayerState {
    id: string;
    source: string;
    x: number;
    y: number;
    width: number;
    height: number;
    opacity: number;
    scaleX: number;
    scaleY: number;
}

export type PlayerState = "idle" | "loading" | "playing" | "paused" | "stopped";

export interface EngineEvents {
    onReady?: () => void;
    onFrame?: (frame: number) => void;
    onStateChange?: (state: PlayerState) => void;
    onError?: (error: string) => void;
    onWebLayerRender?: (layerState: WebLayerState, currentFrame: number) => void;
}

export class VidraEngine {
    private canvas: HTMLCanvasElement;
    private ctx2d: CanvasRenderingContext2D;
    private irJson: string | null = null;
    private info: ProjectInfo | null = null;
    private currentFrame: number = 0;
    private animId: number = 0;
    private state: PlayerState = "idle";

    // iframe DOM sandbox nodes used for web layer rendering
    private webLayerElements: Map<string, HTMLIFrameElement> = new Map();
    // A container injected right next to the canvas for overlaying web layers
    private overlayContainer: HTMLDivElement | null = null;

    /**
     * Materialize an `autocaption(...)` layer from host-provided segments.
     *
     * This updates the engine's in-memory IR JSON (no re-compile needed).
     */
    materializeAutoCaptionLayer(layerId: string, segments: CaptionSegment[]) {
        if (!this.irJson) {
            throw new Error("Engine IR not loaded");
        }
        const segmentsJson = JSON.stringify(segments);
        this.irJson = materialize_autocaption_layer(this.irJson, layerId, segmentsJson);
    }

    /**
     * Apply a background-removal patch to an image layer.
     *
     * The caller provides the PNG-with-alpha bytes and a new asset id.
     */
    applyRemoveBackgroundPatch(layerId: string, newAssetId: string, pngBytes: Uint8Array) {
        if (!this.irJson) {
            throw new Error("Engine IR not loaded");
        }
        load_image_asset(newAssetId, pngBytes);
        this.irJson = apply_remove_background_patch(this.irJson, layerId, newAssetId);
    }
    private lastFrameTime: number = 0;
    private events: EngineEvents;
    private wasmReady: boolean = false;

    constructor(canvas: HTMLCanvasElement, events: EngineEvents = {}) {
        this.canvas = canvas;
        this.ctx2d = canvas.getContext("2d")!;
        this.events = events;
    }

    /** Initialize the WASM module. Must be called once before any rendering. */
    async init(): Promise<void> {
        await init();
        this.wasmReady = true;
        this.setState("idle");
        this.events.onReady?.();
    }

    /** Get the WASM engine version. */
    getVersion(): string {
        return version();
    }

    /** Load a VidraScript source string, compile it, and prepare for rendering. */
    loadSource(source: string): ProjectInfo {
        if (!this.wasmReady) throw new Error("Engine not initialized. Call init() first.");
        this.setState("loading");

        this.irJson = parse_and_compile(source);
        const rawInfo = get_project_info(this.irJson);
        this.info = JSON.parse(rawInfo) as ProjectInfo;

        // Size the canvas to match the project
        this.canvas.width = this.info.width;
        this.canvas.height = this.info.height;

        this.currentFrame = 0;
        this.setState("paused");
        this.renderCurrentFrame();

        return this.info;
    }

    /** Load pre-compiled IR JSON directly. */
    loadIR(irJson: string): ProjectInfo {
        if (!this.wasmReady) throw new Error("Engine not initialized. Call init() first.");
        this.setState("loading");

        this.irJson = irJson;
        const rawInfo = get_project_info(irJson);
        this.info = JSON.parse(rawInfo) as ProjectInfo;

        this.canvas.width = this.info.width;
        this.canvas.height = this.info.height;

        this.currentFrame = 0;
        this.setState("paused");
        this.renderCurrentFrame();

        return this.info;
    }

    /**
     * Load a Project built with the `@sansavision/vidra-sdk` builder API.
     *
     * Accepts any object that has either:
     * - `.toJSONString()` → returns IR JSON string
     * - `.toJSON()` → returns IR object (will be JSON.stringify'd)
     *
     * @example
     * ```ts
     * import { Project, Scene, Layer } from "@sansavision/vidra-player";
     * const project = new Project(1920, 1080, 60);
     * project.addScene(new Scene("s1", 3).addLayer(new Layer("bg").solid("#1a1a2e")));
     * engine.loadProject(project);
     * ```
     */
    loadProject(project: { toJSONString?: () => string; toJSON?: () => unknown }): ProjectInfo {
        let irJson: string;
        if (typeof project.toJSONString === "function") {
            irJson = project.toJSONString();
        } else if (typeof project.toJSON === "function") {
            irJson = JSON.stringify(project.toJSON());
        } else {
            // Last resort — try to stringify the entire object
            irJson = JSON.stringify(project);
        }
        return this.loadIR(irJson);
    }

    /** Load an image asset into the WASM renderer cache. */
    async loadImageAsset(assetId: string, url: string): Promise<void> {
        const response = await fetch(url);
        const buffer = await response.arrayBuffer();
        load_image_asset(assetId, new Uint8Array(buffer));
    }

    /** Start playback from the current frame. */
    play(): void {
        if (!this.irJson || !this.info) return;
        this.setState("playing");
        this.lastFrameTime = performance.now();
        this.tick();
    }

    /** Pause playback. */
    pause(): void {
        this.setState("paused");
        cancelAnimationFrame(this.animId);
    }

    /** Stop playback and reset to frame 0. */
    stop(): void {
        cancelAnimationFrame(this.animId);
        this.currentFrame = 0;
        this.setState("stopped");
        this.renderCurrentFrame();
    }

    /** Seek to a specific frame. */
    seekToFrame(frame: number): void {
        this.currentFrame = Math.max(0, Math.min(frame, (this.info?.totalFrames ?? 1) - 1));
        this.renderCurrentFrame();
        this.events.onFrame?.(this.currentFrame);
    }

    /** Seek to a specific time in seconds. */
    seekToTime(seconds: number): void {
        if (!this.info) return;
        const frame = Math.floor(seconds * this.info.fps);
        this.seekToFrame(frame);
    }

    /** Get the current frame index. */
    getCurrentFrame(): number {
        return this.currentFrame;
    }

    /** Get the current time in seconds. */
    getCurrentTime(): number {
        if (!this.info) return 0;
        return this.currentFrame / this.info.fps;
    }

    /** Get the player state. */
    getState(): PlayerState {
        return this.state;
    }

    /** Get project info. */
    getProjectInfo(): ProjectInfo | null {
        return this.info;
    }

    // ── Internal ─────────────────────────────────────────────────

    private setState(state: PlayerState): void {
        this.state = state;
        this.events.onStateChange?.(state);
    }

    private tick = (): void => {
        if (this.state !== "playing" || !this.info) return;

        const now = performance.now();
        const frameDurationMs = 1000 / this.info.fps;

        if (now - this.lastFrameTime >= frameDurationMs) {
            this.renderCurrentFrame();
            this.events.onFrame?.(this.currentFrame);

            this.currentFrame++;
            if (this.currentFrame >= this.info.totalFrames) {
                this.currentFrame = 0; // Loop
            }
            this.lastFrameTime = now;
        }

        this.animId = requestAnimationFrame(this.tick);
    };

    private renderCurrentFrame(): void {
        if (!this.irJson || !this.info) return;

        try {
            const rgba = render_frame(this.irJson, this.currentFrame);
            const imageData = new ImageData(
                new Uint8ClampedArray(rgba),
                this.info.width,
                this.info.height
            );
            this.ctx2d.putImageData(imageData, 0, 0);

            // Handle Web layers
            try {
                const webStateJson = get_web_layers_state(this.irJson, this.currentFrame);
                const webLayers: WebLayerState[] = JSON.parse(webStateJson);
                for (const layer of webLayers) {
                    this.renderWebLayerInBrowser(layer, this.currentFrame);
                }
            } catch (layerErr) {
                console.warn("[vidra-player] Failed to render web layers for frame", layerErr);
            }
        } catch (e) {
            this.events.onError?.(String(e));
        }
    }

    /**
     * Renders a web layer within the browser by creating a sandboxed <iframe>
     * and positioning it correctly on top of the main canvas.
     */
    private renderWebLayerInBrowser(layer: WebLayerState, frame: number): void {
        // Yield out to the consumer if they desire a custom rendering strategy (e.g. html2canvas/rasterization)
        if (this.events.onWebLayerRender) {
            this.events.onWebLayerRender(layer, frame);
            return;
        }

        // Default DOM-overlay rendering strategy
        if (!this.overlayContainer) {
            const parent = this.canvas.parentElement;
            if (parent) {
                this.overlayContainer = document.createElement("div");
                this.overlayContainer.style.position = "absolute";
                this.overlayContainer.style.top = `${this.canvas.offsetTop}px`;
                this.overlayContainer.style.left = `${this.canvas.offsetLeft}px`;
                this.overlayContainer.style.width = `${this.canvas.offsetWidth}px`;
                this.overlayContainer.style.height = `${this.canvas.offsetHeight}px`;
                this.overlayContainer.style.pointerEvents = "none";
                this.overlayContainer.style.overflow = "hidden";
                parent.insertBefore(this.overlayContainer, this.canvas.nextSibling);

                // Update overlay on window resize to track canvas.
                window.addEventListener("resize", () => {
                    if (this.overlayContainer) {
                        this.overlayContainer.style.top = `${this.canvas.offsetTop}px`;
                        this.overlayContainer.style.left = `${this.canvas.offsetLeft}px`;
                        this.overlayContainer.style.width = `${this.canvas.offsetWidth}px`;
                        this.overlayContainer.style.height = `${this.canvas.offsetHeight}px`;
                    }
                });
            } else {
                return; // Nothing to mount to!
            }
        }

        let iframe = this.webLayerElements.get(layer.id);
        if (!iframe) {
            iframe = document.createElement("iframe");
            iframe.sandbox.add("allow-scripts", "allow-same-origin");
            iframe.style.position = "absolute";
            iframe.style.border = "none";
            // Important: transparent so we can see the canvas below.
            iframe.style.backgroundColor = "transparent";
            iframe.src = layer.source;

            this.overlayContainer.appendChild(iframe);
            this.webLayerElements.set(layer.id, iframe);
        }

        // Compute scaling ratio from the canvas's current CSS dimensions vs its native resolution
        const scaleXRatio = this.canvas.offsetWidth / this.canvas.width;
        const scaleYRatio = this.canvas.offsetHeight / this.canvas.height;

        // Apply physical transform tracking
        iframe.style.width = `${layer.width}px`;
        iframe.style.height = `${layer.height}px`;
        iframe.style.left = `${layer.x * scaleXRatio}px`;
        iframe.style.top = `${layer.y * scaleYRatio}px`;
        iframe.style.opacity = layer.opacity.toString();

        // Use CSS transform to scale it exactly as WASM expects
        iframe.style.transformOrigin = "top left";
        iframe.style.transform = `scale(${layer.scaleX * scaleXRatio}, ${layer.scaleY * scaleYRatio})`;

        // Inject the __vidra_advance_frame protocol payload to the sandboxed document
        iframe.contentWindow?.postMessage({
            type: "vidra_frame",
            frame: frame,
            time: frame / (this.info?.fps || 30),
            fps: this.info?.fps || 30
        }, "*");
    }
}
