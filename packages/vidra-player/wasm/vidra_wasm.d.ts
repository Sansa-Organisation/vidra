/* tslint:disable */
/* eslint-disable */

/**
 * Get project metadata from IR JSON.
 *
 * Returns a JSON string: { width, height, fps, totalFrames, totalDuration, sceneCount }
 */
export function get_project_info(ir_json: string): string;

/**
 * Initialize the WASM module. Call this once before rendering.
 */
export function init(): void;

/**
 * Load an image asset (as raw bytes) into the renderer cache.
 *
 * Call this before rendering frames that reference the asset.
 */
export function load_image_asset(asset_id: string, data: Uint8Array): void;

/**
 * Parse VidraScript source and compile to IR JSON.
 *
 * Returns a JSON string representing the project IR.
 * Throws a JS error if parsing fails.
 */
export function parse_and_compile(source: string): string;

/**
 * Render a single frame and return RGBA pixel data.
 *
 * Returns a `Vec<u8>` of length `width * height * 4`.
 */
export function render_frame(ir_json: string, frame_index: number): Uint8Array;

/**
 * Render a single frame from VidraScript source directly.
 *
 * Convenience method that combines parse + compile + render.
 */
export function render_frame_from_source(source: string, frame_index: number): Uint8Array;

/**
 * Get the version string.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_project_info: (a: number, b: number) => [number, number, number, number];
    readonly load_image_asset: (a: number, b: number, c: number, d: number) => void;
    readonly parse_and_compile: (a: number, b: number) => [number, number, number, number];
    readonly render_frame: (a: number, b: number, c: number) => [number, number, number, number];
    readonly render_frame_from_source: (a: number, b: number, c: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly init: () => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
