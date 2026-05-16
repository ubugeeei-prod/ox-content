import { describe, expect, it } from "vite-plus/test";
import { generateVirtualModule } from "./index";
import type { ResolvedOptions } from "./types";

function importRuntime(base: string) {
  const options = {
    base,
    ssg: { enabled: false },
    docs: false,
    search: { enabled: false },
    i18n: false,
  } as ResolvedOptions;
  const code = generateVirtualModule("runtime", options);
  return import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);
}

describe("runtime helpers", () => {
  it("adds and removes the configured base for local paths", async () => {
    const runtime = await importRuntime("docs");

    expect(runtime.base).toBe("/docs/");
    expect(runtime.withBase("/guide")).toBe("/docs/guide");
    expect(runtime.withBase("guide")).toBe("/docs/guide");
    expect(runtime.withoutBase("/docs/guide")).toBe("/guide");
    expect(runtime.withoutBase("/docs")).toBe("/");
  });

  it("leaves external and hash-only links untouched", async () => {
    const runtime = await importRuntime("/docs/");

    expect(runtime.withBase("https://example.com/a")).toBe("https://example.com/a");
    expect(runtime.withBase("mailto:team@example.com")).toBe("mailto:team@example.com");
    expect(runtime.withBase("#section")).toBe("#section");
    expect(runtime.withBase("javascript:alert(1)")).toBe("/docs/javascript:alert(1)");
    expect(runtime.withoutBase("//cdn.example.com/app.js")).toBe("//cdn.example.com/app.js");
  });
});
