// ─── @sansavision/vidra-web-capture ─────────────────────────────────
//
// This module provides the bridge between user-authored web content
// (React components, vanilla HTML pages) and the Vidra capture harness.
//
// When running inside Vidra's capture harness (Playwright or CDP), the
// global `window.__vidra` object is injected. This class reads from it
// and provides typed access. When running standalone (normal browser),
// it gracefully degrades to real-time defaults.

/**
 * Shape of the `window.__vidra` bridge object injected by the
 * capture harness (`capture.js`).
 */
export interface VidraBridge {
    /** Whether the page is currently being captured by Vidra. */
    capturing: boolean;
    /** Current logical frame number (0-based). */
    frame: number;
    /** Current logical time in seconds. */
    time: number;
    /** Timeline frames-per-second. */
    fps: number;
    /** Arbitrary key/value variables passed from the project IR. */
    vars: Record<string, number>;
    /**
     * Emit a value back to the Vidra capture engine.
     * Useful for communicating computed values (e.g. layout metrics)
     * from the web content back to the video project.
     */
    emit: (key: string, value: unknown) => void;
}

declare global {
    interface Window {
        __vidra?: VidraBridge;
        __vidra_advance_frame?: () => void;
    }
}

/**
 * Return the current Vidra bridge state, or sensible defaults when
 * not running inside a capture harness.
 */
export interface VidraState {
    /** Whether we are inside the Vidra capture harness. */
    capturing: boolean;
    /** Current frame number (0 when standalone). */
    frame: number;
    /** Current time in seconds (real clock when standalone). */
    time: number;
    /** Timeline FPS (60 when standalone). */
    fps: number;
    /** Project variables (empty object when standalone). */
    vars: Record<string, number>;
    /**
     * Emit a value back to the engine. No-op when standalone.
     */
    emit: (key: string, value: unknown) => void;
}

// ─── VidraCapture (vanilla JS) ──────────────────────────────────────

/**
 * A lightweight, framework-agnostic class for reading the Vidra capture
 * bridge and emitting values back.
 *
 * @example
 * ```js
 * const capture = new VidraCapture();
 * const { frame, time, fps, vars, capturing } = capture.getState();
 * console.log(capturing ? `Capturing frame ${frame}` : 'Standalone mode');
 * capture.emit('layout_height', document.body.scrollHeight);
 * ```
 */
export class VidraCapture {
    private startTime: number;

    constructor() {
        this.startTime = typeof performance !== 'undefined' ? performance.now() : Date.now();
    }

    /**
     * Return the current bridge state. If `window.__vidra` exists,
     * values come from the harness. Otherwise, real-time defaults are
     * returned so the page works normally in a browser.
     */
    getState(): VidraState {
        const bridge = typeof window !== 'undefined' ? window.__vidra : undefined;

        if (bridge && bridge.capturing) {
            return {
                capturing: true,
                frame: bridge.frame,
                time: bridge.time,
                fps: bridge.fps,
                vars: bridge.vars ?? {},
                emit: bridge.emit ?? (() => { }),
            };
        }

        // Graceful degradation — standalone mode
        const elapsed = ((typeof performance !== 'undefined' ? performance.now() : Date.now()) - this.startTime) / 1000;
        return {
            capturing: false,
            frame: 0,
            time: elapsed,
            fps: 60,
            vars: {},
            emit: () => { },
        };
    }

    /**
     * Convenience: emit a value to the capture engine.
     * Safe to call even when not in a capture harness (no-op).
     */
    emit(key: string, value: unknown): void {
        this.getState().emit(key, value);
    }

    /**
     * Returns `true` if the page is currently being captured by Vidra.
     */
    isCapturing(): boolean {
        return typeof window !== 'undefined' && !!window.__vidra?.capturing;
    }
}

export default VidraCapture;
