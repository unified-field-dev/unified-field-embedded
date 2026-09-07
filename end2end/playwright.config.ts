import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  // Host suite hydrates a large WASM graph; allow long Orbital boot waits.
  timeout: 300_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  retries: 0,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:3000",
    actionTimeout: 30_000,
    navigationTimeout: 60_000,
    ...devices["Desktop Chrome"],
  },
});
