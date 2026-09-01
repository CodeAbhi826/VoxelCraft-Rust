/* tslint:disable */
/* eslint-disable */

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start: () => void;
    readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_bundle_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_bundle_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
    readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
    readonly wgpu_render_bundle_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue__core_f0fd674eaa06beef___result__Result_____wasm_bindgen_9db9684516082e64___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___js_sys_71a3917a39f159cf___Array__web_sys_d980ed5802a6f028___features__gen_ResizeObserver__ResizeObserver______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___js_sys_71a3917a39f159cf___Array______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1__5: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___js_sys_71a3917a39f159cf___Array______true__6: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__7: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1__8: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1__9: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1__10: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke___wasm_bindgen_9db9684516082e64___JsValue______true__1__11: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_9db9684516082e64___convert__closures_____invoke_______true_: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
