// ─── Vidra Engine — WASM Loader & Render Loop ─────────────────────
// This module loads the WASM module, manages the <canvas>, and drives
// the requestAnimationFrame render loop.

import init, {
    parse_and_compile,
    render_frame,
    render_frame_from_source,
    get_project_info,
    load_image_asset,
    version,
} from "../wasm/vidra_wasm.js";

export interface ProjectInfo {
    width: number;
    height: number;
    fps: number;
    totalFrames: number;
    totalDuration: number;
    sceneCount: number;
}

export type PlayerState = "idle" | "loading" | "playing" | "paused" | "stopped";

export interface EngineEvents {
    onReady?: () => void;
    onFrame?: (frame: number) => void;
    onStateChange?: (state: PlayerState) => void;
    onError?: (error: string) => void;
}

export class VidraEngine {
    private canvas: HTMLCanvasElement;
    private ctx2d: CanvasRenderingContext2D;
    private irJson: string | null = null;
    private info: ProjectInfo | null = null;
    private currentFrame: number = 0;
    private animId: number = 0;
    private state: PlayerState = "idle";
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
        } catch (e) {
            this.events.onError?.(String(e));
        }
    }
}
