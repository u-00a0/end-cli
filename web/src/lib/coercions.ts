export function asRecord(value: unknown): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return {};
  }
  return value as Record<string, unknown>;
}

export function asString(value: unknown, fallback = ''): string {
  if (typeof value !== 'string') {
    return fallback;
  }
  return value;
}

export function asInt(value: unknown, fallback = 0): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    return fallback;
  }
  return Math.max(0, Math.round(value));
}
