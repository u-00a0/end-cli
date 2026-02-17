import { parse as parseToml, stringify as stringifyToml } from 'smol-toml';
import { asInt, asRecord, asString } from './coercions';
import type { AicDraft, DraftPriceRow, DraftSupplyRow, OutpostDraft } from './types';

function parseSupply(map: Record<string, unknown>): DraftSupplyRow[] {
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
  return {
    key: asString(record.key),
    en: asString(record.en),
    zh: asString(record.zh),
    moneyCapPerHour: asInt(record.money_cap_per_hour),
    prices: parsePrices(asRecord(record.prices))
  };
}

function cleanDraft(draft: AicDraft): AicDraft {
  return {
    externalPowerConsumptionW: asInt(draft.externalPowerConsumptionW),
    supply: draft.supply
      .filter((row) => row.itemKey.trim().length > 0)
      .map((row) => ({ itemKey: row.itemKey.trim(), value: asInt(row.value) })),
    outposts: draft.outposts
      .filter((outpost) => outpost.key.trim().length > 0)
      .map((outpost) => ({
        key: outpost.key.trim(),
        en: outpost.en.trim(),
        zh: outpost.zh.trim(),
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
    externalPowerConsumptionW: asInt(parsed.external_power_consumption_w),
    supply: parseSupply(asRecord(parsed.supply_per_min)),
    outposts: Array.isArray(parsed.outposts) ? parsed.outposts.map(parseOutpost) : []
  });
}

export function buildAicToml(draft: AicDraft): string {
  const cleaned = cleanDraft(draft);

  const supplyPerMin = Object.fromEntries(
    cleaned.supply
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

    if (outpost.en.length > 0) {
      base.en = outpost.en;
    }
    if (outpost.zh.length > 0) {
      base.zh = outpost.zh;
    }

    return base;
  });

  return stringifyToml({
    external_power_consumption_w: asInt(cleaned.externalPowerConsumptionW),
    supply_per_min: supplyPerMin,
    outposts
  });
}
