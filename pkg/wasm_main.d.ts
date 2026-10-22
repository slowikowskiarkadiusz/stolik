/* tslint:disable */
/* eslint-disable */

export class Stolik {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    /**
     * Reports a key change. `player` is 0 or 1, `key` is the numeric value of `common::engine::input::key::Key`.
     */
    set_key(player: number, key: number, pressed: boolean): void;
    size(): number;
    /**
     * Advances the engine by `delta_time` seconds and returns RGBA bytes (row-major, y * size + x).
     */
    tick(delta_time: number): Uint8Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_stolik_free: (a: number, b: number) => void;
    readonly _critical_section_1_0_acquire: () => void;
    readonly _embassy_time_now: () => bigint;
    readonly _embassy_time_schedule_wake: (a: bigint, b: number) => void;
    readonly stolik_new: () => number;
    readonly stolik_set_key: (a: number, b: number, c: number, d: number) => void;
    readonly stolik_size: (a: number) => number;
    readonly stolik_tick: (a: number, b: number) => [number, number];
    readonly _critical_section_1_0_release: () => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
