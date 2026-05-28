import { defineConfig, devices } from '@playwright/test';

const PORT = 1420;

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  reporter: process.env.CI ? 'github' : 'list',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    // Spin up Vite's dev server. Tauri's runtime is mocked in each spec via
    // `window.__TAURI_INTERNALS__.invoke`, so the page never tries to talk
    // to a real backend.
    command: 'pnpm dev',
    url: `http://localhost:${PORT}`,
    timeout: 60_000,
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});
