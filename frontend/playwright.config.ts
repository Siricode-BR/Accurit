import { defineConfig, devices } from '@playwright/test';

const ADMIN_URL = process.env.E2E_ADMIN_URL ?? 'http://127.0.0.1:4201';
const CLIENT_URL = process.env.E2E_CLIENT_URL ?? 'http://127.0.0.1:4202';

export default defineConfig({
  testDir: './e2e',
  timeout: 60_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-report' }]],
  use: {
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'admin',
      testMatch: /admin\.spec\.ts/,
      use: { ...devices['Desktop Chrome'], baseURL: ADMIN_URL },
    },
    {
      name: 'client',
      testMatch: /client\.spec\.ts/,
      use: { ...devices['Desktop Chrome'], baseURL: CLIENT_URL },
    },
  ],
});
