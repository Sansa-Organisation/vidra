// ─── Vidra Player — Main Exports ────────────────────────────────
// Engine (WASM-powered renderer)
export { VidraEngine } from "./engine.js";
export type { ProjectInfo, PlayerState, EngineEvents } from "./engine.js";

// AI cache-key helpers (browser-side; matches Rust CLI deterministic keys)
export * from "./ai.js";

// Re-export the entire SDK so devs only need one package
export {
    Project,
    Scene,
    Layer,
    Easing,
    hex,
    rgba,
} from "./sdk.js";
