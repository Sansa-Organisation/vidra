/* tslint:disable */
/* eslint-disable */

/**
 * Chroma subsampling format
 */
export enum ChromaSampling {
    /**
     * Both vertically and horizontally subsampled.
     */
    Cs420 = 0,
    /**
     * Horizontally subsampled.
     */
    Cs422 = 1,
    /**
     * Not subsampled.
     */
    Cs444 = 2,
    /**
     * Monochrome.
     */
    Cs400 = 3,
}

/**
 * Apply a background-removal patch to an image layer.
 *
 * The JS host is responsible for calling the remote API (remove.bg / Clipdrop / etc) and
 * providing the resulting PNG-with-alpha via `load_image_asset(new_asset_id, bytes)`.
 *
 * This function updates the IR to:
 * - swap `image(asset_id)` to the new asset id
 * - remove `effect(removeBackground)` from the layer
 *
 * Returns updated IR JSON.
 */
export function apply_remove_background_patch(ir_json: string, layer_id: string, new_asset_id: string): string;

/**
 * Dispatch a click event at (x, y) for a given frame.
 *
 * Returns a JSON string: { handled: bool, layerId?: string }
 */
export function dispatch_click(ir_json: string, frame_index: number, x: number, y: number): string;

/**
 * Get the last mouse position set via `set_mouse_position`.
 *
 * Returns a JSON string: { x, y }
 */
export function get_mouse_position(): string;

/**
 * Get project metadata from IR JSON.
 *
 * Returns a JSON string: { width, height, fps, totalFrames, totalDuration, sceneCount }
 */
export function get_project_info(ir_json: string): string;

/**
 * Get a numeric runtime state variable.
 *
 * Returns `null` if unset.
 */
export function get_state_var(name: string): any;

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
 * Materialize an `autocaption(...)` layer using caption segments provided by the JS host.
 *
 * This enables web / React Native runtimes to do the network call for transcription and then
 * feed the result into Vidra as a deterministic IR update.
 *
 * - `ir_json`: the project IR JSON string.
 * - `layer_id`: id of the layer whose content is `AutoCaption`.
 * - `segments_json`: JSON array of objects: { start_s, end_s, text }.
 *
 * Returns an updated IR JSON string.
 */
export function materialize_autocaption_layer(ir_json: string, layer_id: string, segments_json: string): string;

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
 * Update the current mouse position (in pixel coordinates) for interactive previews.
 *
 * Note: this currently does not affect rendering output yet; it is exposed as
 * plumbing for upcoming interactive expressions and event handling.
 */
export function set_mouse_position(x: number, y: number): void;

/**
 * Set a numeric runtime state variable used by interactive expressions and event handlers.
 */
export function set_state_var(name: string, value: number): void;

/**
 * Get the version string.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly apply_remove_background_patch: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly dispatch_click: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly get_mouse_position: () => [number, number];
    readonly get_project_info: (a: number, b: number) => [number, number, number, number];
    readonly get_state_var: (a: number, b: number) => any;
    readonly load_image_asset: (a: number, b: number, c: number, d: number) => void;
    readonly materialize_autocaption_layer: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly parse_and_compile: (a: number, b: number) => [number, number, number, number];
    readonly render_frame: (a: number, b: number, c: number) => [number, number, number, number];
    readonly render_frame_from_source: (a: number, b: number, c: number) => [number, number, number, number];
    readonly set_state_var: (a: number, b: number, c: number) => void;
    readonly version: () => [number, number];
    readonly init: () => void;
    readonly set_mouse_position: (a: number, b: number) => void;
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
