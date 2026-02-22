import { describe, expect, it } from 'vitest';
import { buildAicToml, parseAicToml } from './aic';
import type { AicDraft } from './types';

describe('aic toml conversions', () => {
  it('round-trips draft through TOML while preserving normalized data', () => {
    const draft: AicDraft = {
      externalPowerConsumptionW: 322,
      supply: [
        { itemKey: 'IronOre', value: 120 },
        { itemKey: 'CopperOre', value: 80 }
      ],
      consumption: [
        { itemKey: 'IronOre', value: 15 },
        { itemKey: 'Water', value: 8 }
      ],
      outposts: [
        {
          key: 'Refugee_Camp',
          en: 'Refugee Camp',
          zh: '难民营',
          moneyCapPerHour: 17316,
          prices: [
            { itemKey: 'SC Valley Battery', price: 30 },
            { itemKey: 'Origocrust', price: 1 }
          ]
        }
      ]
    };

    const toml = buildAicToml(draft);
    const parsed = parseAicToml(toml);

    expect(parsed.externalPowerConsumptionW).toBe(322);
    expect(parsed.supply).toHaveLength(2);
    expect(parsed.consumption).toHaveLength(2);
    expect(parsed.outposts).toHaveLength(1);
    expect(parsed.outposts[0]?.key).toBe('Refugee_Camp');
    expect(parsed.outposts[0]?.prices).toHaveLength(2);
  });

  it('drops blank keys during cleaning', () => {
    const toml = `external_power_consumption_w = 10
[supply_per_min]
"IronOre" = 5
"" = 99

[external_consumption_per_min]
"IronOre" = 2
"" = 99

[[outposts]]
key = ""
money_cap_per_hour = 100
[outposts.prices]
"Battery" = 2
`;

    const parsed = parseAicToml(toml);

    expect(parsed.supply).toHaveLength(1);
    expect(parsed.consumption).toHaveLength(1);
    expect(parsed.outposts).toHaveLength(0);
  });
});
