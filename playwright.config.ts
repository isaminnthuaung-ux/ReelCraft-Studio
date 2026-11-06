import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/ui', timeout: 60000,
  use: { baseURL: 'http://127.0.0.1:1420', viewport: { width: 1500, height: 1000 }, channel: 'msedge', screenshot: 'only-on-failure' },
  webServer: { command: 'node node_modules/vite/bin/vite.js --host 127.0.0.1 --port 1420 --strictPort', url: 'http://127.0.0.1:1420', reuseExistingServer: true },
});
