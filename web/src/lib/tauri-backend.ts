import type { Backend } from './backend';
import type { BootstrapPayload, LangTag, SolvePayload } from './types';

export function createTauriBackend(): Backend {
  let invokePromise: Promise<typeof import('@tauri-apps/api/core')['invoke']> | null = null;

  async function getInvoke() {
    if (!invokePromise) {
      invokePromise = import('@tauri-apps/api/core').then((module) => module.invoke);
    }

    return invokePromise;
  }

  return {
    async loadBootstrap(lang: LangTag): Promise<BootstrapPayload> {
      const invoke = await getInvoke();
      return invoke<BootstrapPayload>('cmd_bootstrap', { lang });
    },

    async solveScenario(lang: LangTag, aicToml: string): Promise<SolvePayload> {
      const invoke = await getInvoke();
      return invoke<SolvePayload>('cmd_solve', { lang, aicToml });
    },

    async warmup(): Promise<void> {}
  };
}
