import { expect, test } from '@playwright/test';

test('workspace boots and auto solve produces result panels', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: /Solver Inputs|求解输入/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Plan Summary|方案评估/ })).toBeVisible();

  const externalPowerInput = page.locator('#external-power');
  await expect(externalPowerInput).toBeVisible();
  await externalPowerInput.fill('321');

  await expect(page.getByText(/Revenue \/ min|收益 \/ min/)).toBeVisible({ timeout: 60_000 });
  await expect(page.getByText(/Flow Map|物流图/)).toBeVisible();
});
