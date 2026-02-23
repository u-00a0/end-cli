import { parse as parseToml, stringify as stringifyToml } from 'smol-toml';
import { asInt, asRecord, asString } from './coercions';
import type {
  AicDraft,
  DraftPriceRow,
  OutpostDraft,
  ScenarioRegion,
  Stage2ConfigDraft,
  Stage2ObjectiveMode
} from './types';

function asNonNegativeNumber(value: unknown, fallback: number): number {
  const parsed = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return fallback;
  }
  return parsed;
}

function parseStage2Objective(value: unknown): Stage2ObjectiveMode {
  const raw = typeof value === 'string' ? value.trim() : '';
  if (
    raw === 'min_machines' ||
    raw === 'max_power_slack' ||
    raw === 'max_money_slack' ||
    raw === 'weighted'
  ) {
    return raw;
  }
  return 'min_machines';
}

function parseStage2(raw: unknown): Stage2ConfigDraft {
  const record = asRecord(raw);
  return {
    objective: parseStage2Objective(record.objective),
    alpha: asNonNegativeNumber(record.alpha, 1),
    beta: asNonNegativeNumber(record.beta, 1),
    gamma: asNonNegativeNumber(record.gamma, 1)
  };
}

function parseRegion(value: unknown): ScenarioRegion {
  const raw = typeof value === 'string' ? value.trim() : '';
  if (raw === 'fourth_valley' || raw === 'wuling') {
    return raw;
  }
  return 'wuling';
}

function parseItemFlowRows(map: Record<string, unknown>): { itemKey: string; value: number }[] {
  return Object.entries(map)
    .map(([itemKey, value]) => ({
      itemKey,
      value: asInt(value)
    }))
    .filter((row) => row.itemKey.trim().length > 0)
    .sort((a, b) => a.itemKey.localeCompare(b.itemKey));
}

function parsePrices(map: Record<string, unknown>): DraftPriceRow[] {
  return Object.entries(map)
    .map(([itemKey, value]) => ({
      itemKey,
      price: asInt(value)
    }))
    .filter((row) => row.itemKey.trim().length > 0)
    .sort((a, b) => a.itemKey.localeCompare(b.itemKey));
}

function parseOutpost(raw: unknown): OutpostDraft {
  const record = asRecord(raw);
  const zh = asString(record.zh).trim();
  const en = asString(record.en).trim();
  const name = asString(record.name).trim();
  return {
    key: asString(record.key),
    name: name || zh || en,
    moneyCapPerHour: asInt(record.money_cap_per_hour),
    prices: parsePrices(asRecord(record.prices))
  };
}

function cleanDraft(draft: AicDraft): AicDraft {
  return {
    region: draft.region === 'fourth_valley' ? 'fourth_valley' : 'wuling',
    externalPowerConsumptionW: asInt(draft.externalPowerConsumptionW),
    stage2: {
      objective: parseStage2Objective(draft.stage2.objective),
      alpha: asNonNegativeNumber(draft.stage2.alpha, 1),
      beta: asNonNegativeNumber(draft.stage2.beta, 1),
      gamma: asNonNegativeNumber(draft.stage2.gamma, 1)
    },
    supply: draft.supply
      .filter((row) => row.itemKey.trim().length > 0)
      .map((row) => ({ itemKey: row.itemKey.trim(), value: asInt(row.value) })),
    consumption: draft.consumption
      .filter((row) => row.itemKey.trim().length > 0)
      .map((row) => ({ itemKey: row.itemKey.trim(), value: asInt(row.value) })),
    outposts: draft.outposts
      .filter((outpost) => outpost.key.trim().length > 0)
      .map((outpost) => ({
        key: outpost.key.trim(),
        name: outpost.name.trim(),
        moneyCapPerHour: asInt(outpost.moneyCapPerHour),
        prices: outpost.prices
          .filter((row) => row.itemKey.trim().length > 0)
          .map((row) => ({ itemKey: row.itemKey.trim(), price: asInt(row.price) }))
      }))
  };
}

export function parseAicToml(tomlText: string): AicDraft {
  const parsed = parseToml(tomlText) as Record<string, unknown>;
  return cleanDraft({
    region: parseRegion(parsed.region),
    externalPowerConsumptionW: asInt(parsed.external_power_consumption_w),
    stage2: parseStage2(parsed.stage2),
    supply: parseItemFlowRows(asRecord(parsed.supply_per_min)),
    consumption: parseItemFlowRows(asRecord(parsed.external_consumption_per_min)),
    outposts: Array.isArray(parsed.outposts) ? parsed.outposts.map(parseOutpost) : []
  });
}

export function buildAicToml(draft: AicDraft): string {
  const cleaned = cleanDraft(draft);
  const stage2: Record<string, unknown> = {
    objective: cleaned.stage2.objective
  };
  if (cleaned.stage2.objective === 'weighted') {
    stage2.alpha = cleaned.stage2.alpha;
    stage2.beta = cleaned.stage2.beta;
    stage2.gamma = cleaned.stage2.gamma;
  }

  const supplyPerMin = Object.fromEntries(
    cleaned.supply
      .filter((row) => row.value > 0)
      .map((row) => [row.itemKey, asInt(row.value)])
  );
  const externalConsumptionPerMin = Object.fromEntries(
    cleaned.consumption
      .filter((row) => row.value > 0)
      .map((row) => [row.itemKey, asInt(row.value)])
  );

  const outposts = cleaned.outposts.map((outpost) => {
    const prices = Object.fromEntries(outpost.prices.map((row) => [row.itemKey, asInt(row.price)]));

    const base: Record<string, unknown> = {
      key: outpost.key,
      money_cap_per_hour: asInt(outpost.moneyCapPerHour),
      prices
    };

    if (outpost.name.length > 0) {
      base.zh = outpost.name;
    }

    return base;
  });

  return stringifyToml({
    region: cleaned.region,
    external_power_consumption_w: asInt(cleaned.externalPowerConsumptionW),
    stage2,
    supply_per_min: supplyPerMin,
    external_consumption_per_min: externalConsumptionPerMin,
    outposts
  });
}
