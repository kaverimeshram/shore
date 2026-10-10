import { defineConfig, devices } from "@playwright/test";
// PW_BASE_URL runs the same suite against a deployed copy (e.g. https://shore.pages.dev); otherwise a local static server.
const remote = process.env.PW_BASE_URL;
export default defineConfig({
  testDir: "tests",
  timeout: 60_000,
  use: { baseURL: remote || "http://127.0.0.1:4173" },
  webServer: remote ? undefined : { command: "python3 -m http.server 4173 --bind 127.0.0.1", port: 4173, reuseExistingServer: true },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"], viewport: { width: 1360, height: 900 } } },
    { name: "mobile", use: { ...devices["Pixel 7"] } },
  ],
});
