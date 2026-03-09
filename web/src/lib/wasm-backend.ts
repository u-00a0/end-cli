import type { Backend } from './backend';
import type { BootstrapPayload, LangTag, SolvePayload } from './types';
import { warmupWasmWorker, loadBootstrap, solveScenario } from './wasm';

/**
 * WASM backend — delegates to the existing Web Worker + Emscripten WASM pipeline.
 * This is the default backend for browser deployments (no Tauri).
 */
export function createWasmBackend(): Backend {
  return {
    loadBootstrap(lang: LangTag): Promise<BootstrapPayload> {
      return loadBootstrap(lang);
    },

    solveScenario(lang: LangTag, aicToml: string): Promise<SolvePayload> {
      return solveScenario(lang, aicToml);
    },

    warmup(): Promise<void> {
      return warmupWasmWorker();
    },
  };
}
