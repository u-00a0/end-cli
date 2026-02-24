import type { SolvePayload } from './types';

export type SolveOkState = {
  status: 'ok';
  payload: SolvePayload;
  elapsedMs: number;
};

export type SolveState =
  | { status: 'idle' }
  | { status: 'pending'; previousOk: SolveOkState | null } // debouncing, not yet started solving
  | { status: 'solving'; startedAt: number; previousOk: SolveOkState | null } // solve in progress
  | SolveOkState
  | { status: 'err'; message: string };

export function isSolveBusy(state: SolveState): boolean {
  return state.status === 'pending' || state.status === 'solving';
}

export function renderedOkState(state: SolveState): SolveOkState | null {
  if (state.status === 'ok') {
    return state;
  }

  if (state.status === 'pending' || state.status === 'solving') {
    return state.previousOk;
  }

  return null;
}

export function errorMessageOf(state: SolveState): string {
  return state.status === 'err' ? state.message : '';
}
