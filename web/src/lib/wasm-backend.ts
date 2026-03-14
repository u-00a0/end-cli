import type { Backend } from './backend';
import type { BootstrapPayload, LangTag, SolvePayload } from './types';
import { warmupWasmWorker, loadBootstrap, solveScenario } from './wasm';

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
    }
  };
}
