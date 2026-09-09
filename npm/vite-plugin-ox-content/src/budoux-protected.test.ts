import { describe, expect, it } from "vite-plus/test";
import { transformBudouxHtml } from "./budoux";

const tags = ["code", "math", "pre", "script", "style", "svg", "textarea"];
const options = {
  enabled: true,
  language: "ja" as const,
  separator: "|",
  parser: { parse: (text: string) => text.split(" ") },
};

describe("BudouX protected block scanning", () => {
  it.each(tags)("preserves %s after expanding Unicode lowercase characters", async (tag) => {
    const html = `İ<p>before text</p><${tag}>protected text</${tag.toUpperCase()}><p>after text</p>`;
    expect(await transformBudouxHtml(html, options)).toBe(
      `İ<p>before|text</p><${tag}>protected text</${tag.toUpperCase()}><p>after|text</p>`,
    );
  });

  it("keeps adjacent, nested, self-closing, and unclosed protected blocks", async () => {
    const html =
      "<code/><p>visible text</p><pre><code>protected text</code></PRE><svg>more text</svg><style>unclosed text";
    expect(await transformBudouxHtml(html, options)).toBe(
      html.replace("visible text", "visible|text"),
    );
  });

  it("does not retain regex state between documents or concurrent calls", async () => {
    const html = "<code>protected text</code><p>visible text</p>";
    const result = await Promise.all(
      Array.from({ length: 20 }, () => transformBudouxHtml(html, options)),
    );
    expect(result).toEqual(Array(20).fill(html.replace("visible text", "visible|text")));
  });

  it("segments only visible text among 2,100 mixed protected blocks", async () => {
    const html = tags
      .map((tag) => `<${tag}>protected text</${tag}><p>visible text</p>`)
      .join("")
      .repeat(300);
    const result = await transformBudouxHtml(html, options);
    expect(result).toBe(html.replaceAll("visible text", "visible|text"));
  });
});
