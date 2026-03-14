import type { BootstrapPayload, LangTag, SolvePayload } from './types';

export interface Backend {
  loadBootstrap(lang: LangTag): Promise<BootstrapPayload>;
  solveScenario(lang: LangTag, aicToml: string): Promise<SolvePayload>;
  warmup(): Promise<void>;
}

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function createBackend(): Promise<Backend> {
  if (isTauri()) {
    const { createTauriBackend } = await import('./tauri-backend');
    return createTauriBackend();
  }

  const { createWasmBackend } = await import('./wasm-backend');
  return createWasmBackend();
}
