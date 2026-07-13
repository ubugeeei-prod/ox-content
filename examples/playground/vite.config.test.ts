import { describe, expect, it } from "vite-plus/test";
import { resolvePlaygroundBase } from "./vite.config";

describe("resolvePlaygroundBase", () => {
  it("targets the GitHub Pages playground directory for production builds", () => {
    expect(resolvePlaygroundBase("production")).toBe("/ox-content/playground/");
  });

  it("keeps the development server rooted at the origin", () => {
    expect(resolvePlaygroundBase("development")).toBe("/");
  });

  it("supports deployment-specific base paths", () => {
    expect(resolvePlaygroundBase("production", "/preview/playground/")).toBe(
      "/preview/playground/",
    );
  });
});
