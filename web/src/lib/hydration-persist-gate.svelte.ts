export interface HydrationPersistGate {
  markHydrated: () => void;
  persistWhenHydrated: () => void;
}

export function createHydrationPersistGate(
  persist: () => void,
): HydrationPersistGate {
  let hasHydrated = $state(false);

  function markHydrated(): void {
    hasHydrated = true;
  }

  function persistWhenHydrated(): void {
    if (!hasHydrated) {
      return;
    }

    persist();
  }

  return {
    markHydrated,
    persistWhenHydrated,
  };
}
