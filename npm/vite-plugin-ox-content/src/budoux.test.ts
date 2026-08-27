import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveBudouxOptions, transformBudouxHtml } from "./budoux";
import { renderMarkdown } from "./render-markdown";
import { transformMarkdown } from "./transform";

const ZWSP = "\u200b";

describe("BudouX segmentation", () => {
  it("segments visible Japanese text at build time without client runtime imports", async () => {
    const result = await renderMarkdown("今日はとても良い天気です。", "/virtual/budoux.md", {
      budoux: true,
      highlight: false,
    });

    expect(result.html).toContain(`今日は${ZWSP}とても${ZWSP}良い${ZWSP}天気です。`);
    expect(result.code).not.toMatch(/\bfrom ["']budoux["']|import\(["']budoux["']\)/);
    expect(result.code).not.toContain("loadDefaultJapaneseParser");
  });

  it("preserves entities, tags, attributes, links, raw blocks, and island payloads", async () => {
    const html = [
      '<h1 title="日本語">日本語の本文です。</h1>',
      "<p>ViteでSSGします &amp; エンティティです。</p>",
      "<p><em>強調の本文です。</em></p>",
      "<ul><li>項目の説明です。</li></ul>",
      "<blockquote>引用の本文です。</blockquote>",
      "<table><tbody><tr><td>表の値です。</td></tr></tbody></table>",
      "<p><code>日本語のコード</code></p>",
      '<pre><code class="language-md">日本語のpre</code></pre>',
      '<script type="application/json">{"label":"日本語"}</script>',
      '<style>.x::before{content:"日本語"}</style>',
      '<a href="/日本語">リンク先の説明です。</a>',
      [
        '<div data-ox-island="Counter" data-ox-props=\'{"label":"日本語"}\'>',
        '<script type="application/json">{"props":{"label":"日本語"}}</script>',
        "<p>スロットの本文です。</p>",
        "</div>",
      ].join(""),
    ].join("");

    const result = await transformBudouxHtml(html, {
      enabled: true,
      language: "ja",
      separator: "|",
    });

    expect(result).toContain('<h1 title="日本語">日本語の|本文です。</h1>');
    expect(result).toContain("<p>Viteで|SSGします &amp; エンティティです。</p>");
    expect(result).toContain("<p><em>強調の|本文です。</em></p>");
    expect(result).toContain("<ul><li>項目の|説明です。</li></ul>");
    expect(result).toContain("<blockquote>引用の|本文です。</blockquote>");
    expect(result).toContain("<table><tbody><tr><td>表の|値です。</td></tr></tbody></table>");
    expect(result).toContain("<p><code>日本語のコード</code></p>");
    expect(result).toContain('<pre><code class="language-md">日本語のpre</code></pre>');
    expect(result).toContain('<script type="application/json">{"label":"日本語"}</script>');
    expect(result).toContain('<style>.x::before{content:"日本語"}</style>');
    expect(result).toContain('<a href="/日本語">リンク先の|説明です。</a>');
    expect(result).toContain('data-ox-props=\'{"label":"日本語"}\'');
    expect(result).toContain(
      '<script type="application/json">{"props":{"label":"日本語"}}</script>',
    );
    expect(result).toContain("<p>スロットの|本文です。</p>");
  });

  it("keeps inline code, fenced code, entities, tables, and MDX island payloads protected", async () => {
    const result = await transformMarkdown(
      [
        "import Alert from './Alert.svelte'",
        "",
        "# 日本語の本文です。",
        "",
        "本文には `日本語のコード` と &amp; エンティティがあります。",
        "",
        '<Alert label="日本語の属性">スロットの本文です。</Alert>',
        "",
        "| 列 | 値 |",
        "| --- | --- |",
        "| 日本語 | 表の値です。 |",
        "",
        "```md",
        "日本語のpre",
        "```",
      ].join("\n"),
      "/virtual/budoux.mdx",
      createDocsResolvedOptions({
        highlight: false,
        mdx: true,
        budoux: { enabled: true, language: "ja", separator: "|" },
      }),
    );

    expect(result.html).toContain('<h1 id="日本語の本文です">日本語の|本文です。</h1>');
    expect(result.html).toContain("<code>日本語のコード</code>");
    expect(result.html).toContain("&#x26; エンティティが|あります。");
    expect(result.html).toContain("<td>日本語</td>");
    expect(result.html).toContain("<td>表の|値です。</td>");
    expect(result.html).toContain('data-ox-island="Alert"');
    expect(result.html).toContain('"label":"日本語の属性"');
    expect(result.html).toContain("<p>スロットの|本文です。</p>");
    expect(result.html).toContain('<pre><code class="language-md">日本語のpre\n</code></pre>');
  });

  it("resolves disabled, default, and custom parser options", async () => {
    expect(resolveBudouxOptions(false)).toEqual({
      enabled: false,
      language: "ja",
      separator: ZWSP,
    });
    expect(resolveBudouxOptions(true)).toEqual({
      enabled: true,
      language: "ja",
      separator: ZWSP,
    });

    const parser = { parse: (text: string) => text.split(" ") };
    expect(resolveBudouxOptions({ parser, separator: "/" })).toEqual({
      enabled: true,
      language: "ja",
      separator: "/",
      parser,
    });
  });
});
