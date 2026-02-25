import { asInt, asRecord, asString } from './coercions';
import type { AicDraft } from './types';

export interface DraftStorageConfig {
  draftStorageKey: string;
}

export interface RestoredLocalState {
  draft: AicDraft | null;
}

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function parseStoredDraft(text: string): AicDraft | null {
  try {
    const root = JSON.parse(text);
    if (typeof root !== 'object' || root === null || Array.isArray(root)) {
      return null;
    }

    const parsed = root as Record<string, unknown>;
    const regionRaw = asString(parsed.region).trim();
    const region = regionRaw === 'fourth_valley' ? 'fourth_valley' : 'wuling';
    const supplyRows = Array.isArray(parsed.supply) ? parsed.supply : [];
    const consumptionRows = Array.isArray(parsed.consumption) ? parsed.consumption : [];
    const outpostRows = Array.isArray(parsed.outposts) ? parsed.outposts : [];
    const stage2Record = asRecord(parsed.stage2);
    const objectiveRaw = asString(stage2Record.objective).trim();
    const objective =
      objectiveRaw === 'max_power_slack' ||
      objectiveRaw === 'max_money_slack' ||
      objectiveRaw === 'weighted'
        ? objectiveRaw
        : 'min_machines';
    const asNonNegativeNumber = (value: unknown): number => {
      const parsedNumber = typeof value === 'number' ? value : Number(value);
      if (!Number.isFinite(parsedNumber) || parsedNumber < 0) {
        return 1;
      }
      return parsedNumber;
    };

    return {
      region,
      externalPowerConsumptionW: asInt(parsed.externalPowerConsumptionW),
      stage2: {
        objective,
        alpha: asNonNegativeNumber(stage2Record.alpha),
        beta: asNonNegativeNumber(stage2Record.beta),
        gamma: asNonNegativeNumber(stage2Record.gamma)
      },
      supply: supplyRows.map((row) => {
        const record = asRecord(row);
        return {
          itemKey: asString(record.itemKey),
          value: asInt(record.value)
        };
      }),
      consumption: consumptionRows.map((row) => {
        const record = asRecord(row);
        return {
          itemKey: asString(record.itemKey),
          value: asInt(record.value)
        };
      }),
      outposts: outpostRows.map((row) => {
        const record = asRecord(row);
        const priceRows = Array.isArray(record.prices) ? record.prices : [];
        const zh = asString(record.zh).trim();
        const en = asString(record.en).trim();
        const name = asString(record.name).trim();
        return {
          key: asString(record.key),
          name: name || zh || en,
          moneyCapPerHour: asInt(record.moneyCapPerHour),
          prices: priceRows.map((priceRow) => {
            const priceRecord = asRecord(priceRow);
            return {
              itemKey: asString(priceRecord.itemKey),
              price: asInt(priceRecord.price)
            };
          })
        };
      })
    };
  } catch {
    return null;
  }
}

export function restoreLocalState(config: DraftStorageConfig): RestoredLocalState {
  const storage = getStorage();
  if (!storage) {
    return {
      draft: null
    };
  }

  const storedDraft = storage.getItem(config.draftStorageKey);
  if (storedDraft === null) {
    return {
      draft: null
    };
  }

  const draft = parseStoredDraft(storedDraft);
  if (!draft) {
    try {
      storage.removeItem(config.draftStorageKey);
    } catch {
      // Ignore write failures in restricted browser modes.
    }
  }

  return {
    draft
  };
}

export function persistDraft(storageKey: string, draft: AicDraft): void {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  try {
    storage.setItem(storageKey, JSON.stringify(draft));
  } catch {
    // Ignore write failures in restricted browser modes.
  }
}

export function persistLeftPaneRatio(storageKey: string, ratio: number): void {
  persistPaneRatio(storageKey, ratio);
}

export function persistRightPaneRatio(storageKey: string, ratio: number): void {
  persistPaneRatio(storageKey, ratio);
}

function persistPaneRatio(storageKey: string, ratio: number): void {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  try {
    storage.setItem(storageKey, String(ratio));
  } catch {
    // Ignore write failures in restricted browser modes.
  }
}
