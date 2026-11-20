import { defineConfig } from '@playwright/test';
import base from './playwright.config';

const url = 'http://127.0.0.1:1485';
export default defineConfig({ ...base,
  testMatch: 'note-export.spec.ts',
  outputDir: '../../target/ticket-185/playwright',
  reporter: [['list']],
  use: { ...base.use, baseURL: url },
  webServer: { ...base.webServer, command: 'pnpm dev --port 1485', url, reuseExistingServer: false },
});
