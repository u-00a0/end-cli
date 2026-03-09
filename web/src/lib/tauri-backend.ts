import type { Backend } from './backend';
import type { BootstrapPayload, LangTag, SolvePayload } from './types';

/**
 * Tauri backend — calls native Rust commands via Tauri IPC.
 * The Rust side runs `end_web::bootstrap` and `end_web::solve_from_aic_toml` directly
 * (native-compiled, no WASM), which is faster and avoids the Emscripten toolchain.
 */
export function createTauriBackend(): Backend {
  // Lazy-import so this module is only pulled in when running inside Tauri.
  let invokePromise: Promise<typeof import('@tauri-apps/api/core')['invoke']> | null = null;

  async function getInvoke() {
    if (!invokePromise) {
      invokePromise = import('@tauri-apps/api/core').then((m) => m.invoke);
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

    async warmup(): Promise<void> {
      // In Tauri mode there is no WASM module to warm up.
      // Optionally we could fire a no-op bootstrap call here to warm the Rust side,
      // but it's fast enough natively that it's not needed.
    },
  };
}
