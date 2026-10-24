import { defineConfig } from '@playwright/test';

const BASE_URL = 'http://127.0.0.1:1420';
const TEST_TIMEOUT_MS = 30_000;
const SERVER_TIMEOUT_MS = 60_000;

export default defineConfig({
  testDir: './e2e',
  timeout: TEST_TIMEOUT_MS,
  fullyParallel: true,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: BASE_URL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    viewport: { width: 1366, height: 768 },
    launchOptions: { executablePath: process.env.PLAYWRIGHT_EXECUTABLE_PATH },
  },
  webServer: {
    command: 'pnpm dev',
    url: BASE_URL,
    reuseExistingServer: false,
    timeout: SERVER_TIMEOUT_MS,
  },
});
