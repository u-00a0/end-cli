import type { BootstrapPayload, LangTag, SolvePayload } from './types';

interface BootstrapRequest {
  id: number;
  kind: 'bootstrap';
  lang: LangTag;
}

interface WarmupRequest {
  id: number;
  kind: 'warmup';
}

interface SolveRequest {
  id: number;
  kind: 'solve';
  lang: LangTag;
  aicToml: string;
}

interface WorkerInitRequest {
  kind: 'init';
  wasmBase: string;
}

interface BootstrapOk {
  id: number;
  kind: 'bootstrap_ok';
  payload: BootstrapPayload;
}

interface BootstrapErr {
  id: number;
  kind: 'bootstrap_err';
  message: string;
}

interface WarmupOk {
  id: number;
  kind: 'warmup_ok';
}

interface WarmupErr {
  id: number;
  kind: 'warmup_err';
  message: string;
}

interface SolveOk {
  id: number;
  kind: 'solve_ok';
  payload: SolvePayload;
}

interface SolveErr {
  id: number;
  kind: 'solve_err';
  message: string;
}

type WorkerResponse = BootstrapOk | BootstrapErr | WarmupOk | WarmupErr | SolveOk | SolveErr;

let solveWorker: Worker | null = null;
let workerRequestId = 1;
let warmupPromise: Promise<void> | null = null;

function deriveAppBasePathname(): string {
  const moduleDir = new URL('.', import.meta.url);
  const appBaseDir = new URL('..', moduleDir);
  const pathname = appBaseDir.pathname;
  return pathname.endsWith('/') ? pathname : `${pathname}/`;
}

const wasmBase = `${deriveAppBasePathname()}wasm/`;

const pendingBootstrap = new Map<
  number,
  {
    resolve: (value: BootstrapPayload) => void;
    reject: (reason?: unknown) => void;
  }
>();

const pendingWarmup = new Map<
  number,
  {
    resolve: () => void;
    reject: (reason?: unknown) => void;
  }
>();

const pendingSolve = new Map<
  number,
  {
    resolve: (value: SolvePayload) => void;
    reject: (reason?: unknown) => void;
  }
>();

class WorkerTransportError extends Error {}

function rejectAllPending(error: Error): void {
  for (const pending of pendingBootstrap.values()) {
    pending.reject(error);
  }
  pendingBootstrap.clear();

  for (const pending of pendingWarmup.values()) {
    pending.reject(error);
  }
  pendingWarmup.clear();

  for (const pending of pendingSolve.values()) {
    pending.reject(error);
  }
  pendingSolve.clear();
}

function nextRequestId(): number {
  const requestId = workerRequestId;
  workerRequestId += 1;
  return requestId;
}

function handleWorkerResponse(response: WorkerResponse): void {
  if (response.kind === 'bootstrap_ok' || response.kind === 'bootstrap_err') {
    const pending = pendingBootstrap.get(response.id);
    if (!pending) {
      return;
    }

    pendingBootstrap.delete(response.id);
    if (response.kind === 'bootstrap_ok') {
      pending.resolve(response.payload);
      return;
    }

    pending.reject(new Error(response.message));
    return;
  }

  if (response.kind === 'warmup_ok' || response.kind === 'warmup_err') {
    const pending = pendingWarmup.get(response.id);
    if (!pending) {
      return;
    }

    pendingWarmup.delete(response.id);
    if (response.kind === 'warmup_ok') {
      pending.resolve();
      return;
    }

    pending.reject(new Error(response.message));
    return;
  }

  const pending = pendingSolve.get(response.id);
  if (!pending) {
    return;
  }

  pendingSolve.delete(response.id);
  if (response.kind === 'solve_ok') {
    pending.resolve(response.payload);
    return;
  }

  pending.reject(new Error(response.message));
}

function getSolveWorker(): Worker {
  if (solveWorker) {
    return solveWorker;
  }

  solveWorker = new Worker(new URL('./solve.worker.ts', import.meta.url), {
    type: 'classic'
  });
  solveWorker.onmessage = (event: MessageEvent<WorkerResponse>) => {
    handleWorkerResponse(event.data);
  };

  solveWorker.onerror = (event: ErrorEvent) => {
    solveWorker = null;
    warmupPromise = null;
    rejectAllPending(new WorkerTransportError(event.message || 'solve worker crashed'));
  };

  try {
    const initRequest: WorkerInitRequest = {
      kind: 'init',
      wasmBase
    };
    solveWorker.postMessage(initRequest);
  } catch (error) {
    solveWorker.terminate();
    solveWorker = null;
    warmupPromise = null;
    throw new WorkerTransportError(error instanceof Error ? error.message : String(error));
  }

  return solveWorker;
}

export function warmupWasmWorker(): Promise<void> {
  if (warmupPromise) {
    return warmupPromise;
  }

  const worker = getSolveWorker();
  const request: WarmupRequest = {
    id: nextRequestId(),
    kind: 'warmup'
  };

  const promise = new Promise<void>((resolve, reject) => {
    pendingWarmup.set(request.id, { resolve, reject });

    try {
      worker.postMessage(request);
    } catch (error) {
      solveWorker = null;
      warmupPromise = null;
      pendingWarmup.delete(request.id);
      reject(new WorkerTransportError(error instanceof Error ? error.message : String(error)));
    }
  });

  warmupPromise = promise.catch((error) => {
    warmupPromise = null;
    throw error;
  });
  return warmupPromise;
}

export function loadBootstrap(lang: LangTag): Promise<BootstrapPayload> {
  const worker = getSolveWorker();
  const request: BootstrapRequest = {
    id: nextRequestId(),
    kind: 'bootstrap',
    lang
  };

  return new Promise<BootstrapPayload>((resolve, reject) => {
    pendingBootstrap.set(request.id, { resolve, reject });

    try {
      worker.postMessage(request);
    } catch (error) {
      solveWorker = null;
      warmupPromise = null;
      pendingBootstrap.delete(request.id);
      reject(new WorkerTransportError(error instanceof Error ? error.message : String(error)));
    }
  });
}

export function solveScenario(lang: LangTag, aicToml: string): Promise<SolvePayload> {
  const worker = getSolveWorker();
  const request: SolveRequest = {
    id: nextRequestId(),
    kind: 'solve',
    lang,
    aicToml
  };

  return new Promise<SolvePayload>((resolve, reject) => {
    pendingSolve.set(request.id, { resolve, reject });

    try {
      worker.postMessage(request);
    } catch (error) {
      solveWorker = null;
      warmupPromise = null;
      pendingSolve.delete(request.id);
      reject(new WorkerTransportError(error instanceof Error ? error.message : String(error)));
    }
  });
}
