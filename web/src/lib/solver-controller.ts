import type { AicDraft, LangTag, SolvePayload } from './types';

export type SolveTrigger = 'auto' | 'manual';

export interface SolveSnapshot {
  draft: AicDraft;
  lang: LangTag;
  isBootstrapping: boolean;
}

interface CreateSolverControllerOptions {
  debounceMs: number;
  getSnapshot: () => SolveSnapshot;
  toToml: (draft: AicDraft) => string;
  solve: (lang: LangTag, toml: string) => Promise<SolvePayload>;
  t: (zh: string, en: string) => string;
  onSolvingChange: (next: boolean) => void;
  onStatusMessage: (next: string) => void;
  onErrorMessage: (next: string) => void;
  onSolved: (payload: SolvePayload, trigger: SolveTrigger) => void;
}

export interface SolverController {
  scheduleAutoSolve: () => void;
  runSolve: (trigger?: SolveTrigger) => Promise<void>;
  dispose: () => void;
  resetSolvedFingerprint: () => void;
}

export function createSolverController(options: CreateSolverControllerOptions): SolverController {
  let autoSolveTimer: number | null = null;
  let autoSolveDirty = false;
  let solveSequence = 0;
  let latestSolveSequence = 0;
  let lastSolvedFingerprint = '';
  let isSolving = false;

  function clearAutoSolveTimer(): void {
    if (autoSolveTimer === null) {
      return;
    }
    window.clearTimeout(autoSolveTimer);
    autoSolveTimer = null;
  }

  function scheduleAutoSolve(): void {
    const snapshot = options.getSnapshot();
    if (snapshot.isBootstrapping) {
      return;
    }

    autoSolveDirty = true;
    clearAutoSolveTimer();
    autoSolveTimer = window.setTimeout(() => {
      autoSolveTimer = null;
      void runSolve('auto');
    }, options.debounceMs);
  }

  async function runSolve(trigger: SolveTrigger = 'manual'): Promise<void> {
    const snapshot = options.getSnapshot();
    if (snapshot.isBootstrapping) {
      return;
    }

    if (isSolving) {
      autoSolveDirty = true;
      return;
    }

    autoSolveDirty = false;

    let toml = '';
    try {
      toml = options.toToml(snapshot.draft);
    } catch (error) {
      options.onErrorMessage(error instanceof Error ? error.message : String(error));
      options.onStatusMessage('');
      return;
    }

    const fingerprint = `${snapshot.lang}\n${toml}`;
    if (fingerprint === lastSolvedFingerprint) {
      if (trigger === 'manual') {
        options.onStatusMessage(options.t('输入未变化，无需重算。', 'Inputs unchanged. Skipped.'));
      }
      return;
    }

    solveSequence += 1;
    const sequence = solveSequence;
    latestSolveSequence = sequence;

    isSolving = true;
    options.onSolvingChange(true);
    options.onErrorMessage('');
    options.onStatusMessage(
      trigger === 'auto' ? options.t('正在自动求解...', 'Auto solving...') : options.t('正在求解...', 'Solving...')
    );

    try {
      const solved = await options.solve(snapshot.lang, toml);
      if (sequence !== latestSolveSequence) {
        return;
      }

      options.onSolved(solved, trigger);
      lastSolvedFingerprint = fingerprint;
      options.onStatusMessage(
        trigger === 'auto'
          ? options.t('自动求解完成。', 'Auto solve completed.')
          : options.t('求解完成。', 'Solve completed.')
      );
    } catch (error) {
      if (sequence !== latestSolveSequence) {
        return;
      }
      options.onErrorMessage(error instanceof Error ? error.message : String(error));
      options.onStatusMessage('');
    } finally {
      if (sequence === latestSolveSequence) {
        isSolving = false;
        options.onSolvingChange(false);
      }

      if (autoSolveDirty) {
        scheduleAutoSolve();
      }
    }
  }

  function dispose(): void {
    clearAutoSolveTimer();
  }

  function resetSolvedFingerprint(): void {
    lastSolvedFingerprint = '';
  }

  return {
    scheduleAutoSolve,
    runSolve,
    dispose,
    resetSolvedFingerprint
  };
}
