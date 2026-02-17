import { expect, test } from '@playwright/test';

const wasmShim = `
(() => {
  const payloadByPtr = new Map();
  let nextPtr = 1;

  const bootstrap = {
    status: 'ok',
    data: {
      defaultAicToml: [
        'external_power_consumption_w = 120',
        '',
        '[supply_per_min]',
        '"IronOre" = 40',
        '',
        '[[outposts]]',
        'key = "Refugee_Camp"',
        'en = "Refugee Camp"',
        'zh = "难民营"',
        'money_cap_per_hour = 12000',
        '[outposts.prices]',
        '"Battery" = 30'
      ].join('\\n'),
      catalog: {
        items: [
          { key: 'IronOre', en: 'Iron Ore', zh: '铁矿' },
          { key: 'Battery', en: 'Battery', zh: '电池' }
        ]
      }
    }
  };

  const solved = {
    status: 'ok',
    data: {
      reportText: 'ok',
      summary: {
        lang: 'en',
        stage1RevenuePerMin: 10,
        stage2RevenuePerMin: 12.34,
        stage2RevenuePerHour: 740,
        totalMachines: 9,
        totalThermalBanks: 1,
        powerGenW: 450,
        powerUseW: 420,
        powerMarginW: 30,
        outposts: [
          {
            key: 'Refugee_Camp',
            name: 'Refugee Camp',
            valuePerMin: 12.34,
            capPerMin: 20,
            ratio: 0.617
          }
        ],
        topSales: [
          {
            outpostKey: 'Refugee_Camp',
            outpostName: 'Refugee Camp',
            itemKey: 'Battery',
            itemName: 'Battery',
            valuePerMin: 12.34
          }
        ],
        facilities: [{ key: 'Assembler', name: 'Assembler', machines: 4 }],
        externalSupplySlack: []
      },
      logisticsGraph: {
        items: [],
        nodes: [],
        edges: []
      }
    }
  };

  globalThis.createEndWebModule = async () => ({
    ccall(ident, returnType, argTypes, args) {
      if (ident === 'end_web_free_c_string') {
        payloadByPtr.delete(args[0]);
        return undefined;
      }

      const envelope =
        ident === 'end_web_bootstrap'
          ? bootstrap
          : ident === 'end_web_solve_from_aic_toml'
            ? solved
            : { status: 'err', error: { message: 'unknown wasm call: ' + ident } };

      const ptr = nextPtr++;
      payloadByPtr.set(ptr, JSON.stringify(envelope));
      return ptr;
    },
    UTF8ToString(ptr) {
      return payloadByPtr.get(ptr) ?? '';
    }
  });
})();
`;

test('workspace boots and auto solve produces result panels', async ({ page }) => {
  await page.route('**/wasm/end_web.js', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/javascript',
      body: wasmShim
    });
  });

  await page.goto('/');

  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  await expect(page.getByText(/Configuration Editor|配置编辑器/)).toBeVisible();
  await expect(page.getByText(/Solver Output|求解结果/)).toBeVisible();

  const externalPowerInput = page.locator('#external-power');
  await expect(externalPowerInput).toBeVisible();
  await externalPowerInput.fill('321');

  await expect(page.getByText(/Revenue \/ min|收益 \/ min/)).toBeVisible({ timeout: 60_000 });
  await expect(page.getByText(/Logistics Graph|物流图/)).toBeVisible();
});
