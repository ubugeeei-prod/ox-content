import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./test/vrt",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: "line",
  expect: {
    toHaveScreenshot: {
      pathTemplate: "{testDir}/{testFileName}-snapshots/{arg}{ext}",
    },
  },
  use: {
    headless: true,
    viewport: { width: 1600, height: 1200 },
    deviceScaleFactor: 1,
    colorScheme: "dark",
    launchOptions: {
      args: ["--font-render-hinting=none", "--disable-lcd-text", "--force-color-profile=srgb"],
    },
  },
});
