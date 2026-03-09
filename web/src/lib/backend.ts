import type { BootstrapPayload, LangTag, SolvePayload } from './types';

/**
 * Backend abstraction — unified interface for both WASM (web) and Tauri (desktop) modes.
 */
export interface Backend {
  loadBootstrap(lang: LangTag): Promise<BootstrapPayload>;
  solveScenario(lang: LangTag, aicToml: string): Promise<SolvePayload>;
  warmup(): Promise<void>;
}

/**
 * Detect whether the app is running inside a Tauri webview.
 */
function isTauri(): boolean {
  return '__TAURI_INTERNALS__' in window;
}

/**
 * Create the appropriate backend for the current runtime environment.
 * - In Tauri desktop mode: uses IPC `invoke()` to call native Rust commands.
 * - In browser mode: uses Emscripten WASM running in a Web Worker.
 */
export async function createBackend(): Promise<Backend> {
  if (isTauri()) {
    const { createTauriBackend } = await import('./tauri-backend');
    return createTauriBackend();
  }
  const { createWasmBackend } = await import('./wasm-backend');
  return createWasmBackend();
}
