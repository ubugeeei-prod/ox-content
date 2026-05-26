import { describe, expect, it } from "vitest";

import { errorHtml, escapeHtml } from "../../internal/preview-html";

describe("escapeHtml", () => {
  it("escapes &, <, and >", () => {
    expect(escapeHtml("a & b < c > d")).toBe("a &amp; b &lt; c &gt; d");
  });

  it("escapes & before introducing new entities so they are not double-encoded", () => {
    expect(escapeHtml("<&>")).toBe("&lt;&amp;&gt;");
  });

  it("returns the input unchanged when there is nothing to escape", () => {
    expect(escapeHtml("plain text")).toBe("plain text");
  });
});

describe("errorHtml", () => {
  it("produces a complete HTML document", () => {
    const html = errorHtml("boom");
    expect(html.startsWith("<!doctype html>")).toBe(true);
    expect(html).toContain("<html");
    expect(html).toContain("</html>");
  });

  it("escapes the message so a malicious payload cannot break out of the card", () => {
    const html = errorHtml("<script>alert(1)</script>");
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
  });

  it("surfaces the recovery hint pointing at oxContent.server.path", () => {
    expect(errorHtml("failed")).toContain("oxContent.server.path");
  });
});
