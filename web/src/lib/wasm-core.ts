import type { ApiEnvelope } from './types';

export interface EndWebModule {
  ccall(
    ident: string,
    returnType: 'number' | 'void',
    argTypes: Array<'string' | 'number'>,
    args: unknown[]
  ): number | undefined;
  UTF8ToString(ptr: number): string;
}

export function callJsonApi<T>(module: EndWebModule, fnName: string, stringArgs: string[]): T {
  const ptr = module.ccall(
    fnName,
    'number',
    stringArgs.map(() => 'string'),
    stringArgs
  );

  if (!ptr || ptr <= 0) {
    throw new Error(`WASM function ${fnName} returned null pointer`);
  }

  try {
    const raw = module.UTF8ToString(ptr);
    const envelope = JSON.parse(raw) as ApiEnvelope<T>;

    if (envelope.status === 'err') {
      throw new Error(envelope.error.message);
    }

    return envelope.data;
  } finally {
    module.ccall('end_web_free_c_string', 'void', ['number'], [ptr]);
  }
}
