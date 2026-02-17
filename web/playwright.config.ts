import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  timeout: 60_000,
  expect: {
    timeout: 10_000
  },
  use: {
    baseURL: 'http://127.0.0.1:4173',
    headless: true
  },
  projects: [
    {
      name: 'firefox',
      use: {
        browserName: 'firefox'
      }
    }
  ],
  webServer: {
    command: 'npm run dev -- --host 127.0.0.1 --port 4173 --strictPort',
    port: 4173,
    timeout: 120_000,
    reuseExistingServer: true
  }
});
