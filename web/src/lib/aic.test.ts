import { describe, expect, it } from 'vitest';
import { buildAicToml, parseAicToml } from './aic';
import type { AicDraft } from './types';

describe('aic toml conversions', () => {
  it('round-trips draft through TOML while preserving normalized data', () => {
    const draft: AicDraft = {
      region: 'fourth_valley',
      externalPowerConsumptionW: 322,
      stage2: {
        objective: 'weighted',
        alpha: 1.5,
        beta: 2,
        gamma: 0.5
      },
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
          name: 'Refugee Camp',
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
    expect(parsed.region).toBe('fourth_valley');
    expect(parsed.stage2.objective).toBe('weighted');
    expect(parsed.stage2.alpha).toBe(1.5);
    expect(parsed.stage2.beta).toBe(2);
    expect(parsed.stage2.gamma).toBe(0.5);
    expect(parsed.supply).toHaveLength(2);
    expect(parsed.consumption).toHaveLength(2);
    expect(parsed.outposts).toHaveLength(1);
    expect(parsed.outposts[0]?.key).toBe('Refugee_Camp');
    expect(parsed.outposts[0]?.name).toBe('Refugee Camp');
    expect(parsed.outposts[0]?.prices).toHaveLength(2);
  });

  it('parses legacy en/zh outpost names into unified name field', () => {
    const toml = `region = "wuling"
external_power_consumption_w = 0

[[outposts]]
key = "Legacy_Outpost"
zh = "旧据点"
en = "Legacy Outpost"
money_cap_per_hour = 10
[outposts.prices]
"Battery" = 2
`;

    const parsed = parseAicToml(toml);

    expect(parsed.outposts).toHaveLength(1);
    expect(parsed.outposts[0]?.name).toBe('旧据点');
    expect(parsed.stage2.objective).toBe('min_machines');
  });

  it('drops blank keys during cleaning', () => {
    const toml = `region = "wuling"
external_power_consumption_w = 10
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
    expect(parsed.region).toBe('wuling');
  });

  it('does not emit weighted coefficients for non-weighted stage2 objective', () => {
    const draft: AicDraft = {
      region: 'wuling',
      externalPowerConsumptionW: 10,
      stage2: {
        objective: 'min_machines',
        alpha: 1.5,
        beta: 2,
        gamma: 3
      },
      supply: [],
      consumption: [],
      outposts: []
    };

    const toml = buildAicToml(draft);

    expect(toml).toContain('[stage2]');
    expect(toml).toContain('objective = "min_machines"');
    expect(toml).not.toContain('alpha =');
    expect(toml).not.toContain('beta =');
    expect(toml).not.toContain('gamma =');
  });
});
