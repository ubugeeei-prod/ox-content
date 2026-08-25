import { describe, expect, it } from "vite-plus/test";
import { transformMarkdown } from "./transform";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  attachTypedHoverPayloads,
  resolveTypedHoverOptions,
  serializeTypedHoverPayload,
} from "./typed-hover";

const TWOSLASH_SNIPPET = ["```ts twoslash", "const value = 1;", "```"].join("\n");

describe("typedHover option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveTypedHoverOptions(undefined).enabled).toBe(false);
    expect(resolveTypedHoverOptions(false).enabled).toBe(false);
    expect(resolveTypedHoverOptions(true).enabled).toBe(true);
    expect(resolveTypedHoverOptions({}).enabled).toBe(true);
    expect(resolveTypedHoverOptions({ enabled: false }).enabled).toBe(false);
    expect(resolveTypedHoverOptions(true).languages).toEqual(["ts", "tsx"]);
  });
});

describe("typedHover transform", () => {
  it("does not attach hover markup when the option is omitted", async () => {
    const result = await transformMarkdown(
      TWOSLASH_SNIPPET,
      "docs/typed-hover-default.md",
      createDocsResolvedOptions({ highlight: false }),
    );

    expect(result.html).not.toContain("ox-typed-hover");
    expect(result.html).not.toContain("ox-typed-hover-token");
    expect(result.html).not.toContain("ox-typed-hover-data");
  });

  it("attaches build-time hover payloads for opted-in TypeScript fences", async () => {
    const result = await transformMarkdown(
      TWOSLASH_SNIPPET,
      "docs/typed-hover-happy.md",
      createDocsResolvedOptions({
        highlight: false,
        typedHover: { enabled: true, languages: ["ts", "tsx"] },
      }),
    );

    expect(result.html).toContain('class="ox-typed-hover"');
    expect(result.html).toContain("ox-typed-hover-token");
    expect(result.html).toContain('tabindex="0"');
    expect(result.html).toContain('type="application/json"');
    expect(result.html).toContain("ox-typed-hover-data");
    expect(result.html).toMatch(/number/);
    expect(result.html).toContain("ox-typed-hover-runtime");
    expect(result.html).not.toContain("createLanguageService");
    expect(result.html).not.toContain("typescript/lib");
  });

  it("skips TypeScript fences that omit the twoslash meta", async () => {
    const result = await transformMarkdown(
      ["```ts", "const value = 1;", "```"].join("\n"),
      "docs/typed-hover-no-meta.md",
      createDocsResolvedOptions({
        highlight: false,
        typedHover: { enabled: true, languages: ["ts", "tsx"] },
      }),
    );

    expect(result.html).toContain("const value = 1;");
    expect(result.html).not.toContain("ox-typed-hover");
  });

  it("leaves inline code spans, JS fences, and unclosed fences alone", async () => {
    const enabled = createDocsResolvedOptions({
      highlight: false,
      typedHover: { enabled: true, languages: ["ts", "tsx"] },
    });

    const inline = await transformMarkdown(
      "Use `const value = 1;` in prose.",
      "docs/typed-hover-inline.md",
      enabled,
    );
    expect(inline.html).toContain("<code>");
    expect(inline.html).not.toContain("ox-typed-hover");

    const jsFence = await transformMarkdown(
      ["```js twoslash", "const value = 1;", "```"].join("\n"),
      "docs/typed-hover-js.md",
      enabled,
    );
    expect(jsFence.html).toContain("const value = 1;");
    expect(jsFence.html).not.toContain("ox-typed-hover");

    await expect(
      transformMarkdown(
        ["# Open", "", "```ts twoslash", "const value = 1;"].join("\n"),
        "docs/typed-hover-unclosed.md",
        enabled,
      ),
    ).resolves.toMatchObject({
      html: expect.stringContaining("value"),
    });
  });

  it("escapes hostile type strings in attached payloads", () => {
    const hostile = "<img src=x onerror=alert(1)></script><script>alert(1)</script>";
    const html = '<pre><code class="language-ts">const value = 1;\n</code></pre>\n';

    const serialized = serializeTypedHoverPayload({
      hovers: [{ start: 6, end: 11, type: hostile }],
    });
    expect(serialized).not.toContain("<img");
    expect(serialized).not.toContain("</script>");
    expect(serialized).toContain("\\u003c");

    const attached = attachTypedHoverPayloads(html, [
      {
        code: "const value = 1;",
        hovers: [{ start: 6, end: 11, type: hostile }],
      },
    ]);

    expect(attached).toContain("ox-typed-hover-token");
    expect(attached).not.toContain("<img");
    expect(attached).not.toMatch(/<\/script>\s*<script>/);
    expect(attached).toContain("\\u003c");
    expect(attached).toContain('tabindex="0"');
    expect(attached).toContain('closest(".ox-code")');
  });
});
