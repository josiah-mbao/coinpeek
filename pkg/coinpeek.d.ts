/* tslint:disable */
/* eslint-disable */

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h8b162bb482474ac4: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h26df45060252b6a7: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h978b21fb76a8fe1c: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hbd3e563b557f5b2e: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h4fd9a8c3b1de36cb: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h60ce205b5c06004d: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__he999e2cc854b0810: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hbd3207cd8d203819: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hf566f87432125b59: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_drop_slice: (a: number, b: number) => void;
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
