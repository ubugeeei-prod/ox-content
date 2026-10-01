import { defineConfig } from "cf/config";

export default defineConfig({
  worker: {
    name: "ox-content-docs",
    compatibilityDate: "2026-10-01",
    workersDev: false,
    previewUrls: false,
    domains: ["ox-content.dev"],
    assets: {
      htmlHandling: "auto-trailing-slash",
      notFoundHandling: "404-page",
    },
  },
});
