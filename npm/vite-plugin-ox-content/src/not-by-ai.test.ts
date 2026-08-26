import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { isReservedBuiltinComponent } from "./plugins/embed-transform";
import { renderMarkdown } from "./render-markdown";
import { resolveNotByAiOptions } from "./not-by-ai-options";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

const LABEL = "Written by human, not by AI";
const HREF = "https://notbyai.fyi";

function options(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}

describe("resolveNotByAiOptions", () => {
  it("omitted => off; true / {} => on with defaults", () => {
    expect(resolveNotByAiOptions(undefined)).toEqual({
      enabled: false,
      label: LABEL,
      href: HREF,
    });
    expect(resolveNotByAiOptions(true).enabled).toBe(true);
    expect(resolveNotByAiOptions({}).enabled).toBe(true);
    expect(resolveNotByAiOptions({ enabled: false }).enabled).toBe(false);
    expect(resolveNotByAiOptions({ label: "Human", href: "https://example.com" })).toEqual({
      enabled: true,
      label: "Human",
      href: "https://example.com",
    });
  });
});

describe("NotByAI transform", () => {
  it("emits identical static markup for .md and .mdx", async () => {
    const source = "Authored <NotByAI />.\n";
    const resolved = options({
      notByAi: { enabled: true, label: LABEL, href: HREF },
    });

    const markdown = await transformMarkdown(source, "/virtual/article.md", {
      ...resolved,
      mdx: false,
    });
    const mdx = await transformMarkdown(source, "/virtual/article.mdx", {
      ...resolved,
      mdx: true,
    });

    for (const result of [markdown, mdx]) {
      expect(result.html).toContain('class="ox-not-by-ai"');
      expect(result.html).toContain(`href="${HREF}"`);
      expect(result.html).toContain(`aria-label="${LABEL}"`);
      expect(result.html).toContain("ox-not-by-ai__badge--light");
      expect(result.html).toContain("ox-not-by-ai__badge--dark");
      expect(result.html).not.toContain('data-ox-island="NotByAI"');
      expect(result.html).not.toContain("<NotByAI");
      expect(result.html).not.toMatch(/<script(?![^>]*type="application\/json")/i);
    }
    expect(markdown.html).toBe(mdx.html);
  });

  it("stays literal when the option is off", async () => {
    const result = await transformMarkdown("<NotByAI />\n", "/virtual/article.md", options());
    expect(result.html).not.toContain("ox-not-by-ai");
    expect(result.html).toMatch(/notbyai/i);
    expect(result.html).not.toContain('data-ox-island="NotByAI"');
  });

  it("reserves NotByAI so MDX island lowering cannot swallow it", () => {
    expect(isReservedBuiltinComponent("NotByAI")).toBe(true);
  });

  it("keeps framework adapters on the same static markup", async () => {
    const result = await renderMarkdown("<NotByAI />\n", "/virtual/article.md", { notByAi: true });
    expect(result.html).toContain('class="ox-not-by-ai"');
    expect(result.html).not.toContain('data-ox-island="NotByAI"');
    expect(result.html).not.toContain("<script");
  });
});
