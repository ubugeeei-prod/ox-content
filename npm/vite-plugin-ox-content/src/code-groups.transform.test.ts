import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformAllPlugins } from "./plugins";
import { resetTabGroupCounter } from "./plugins/tabs";
import { transformMarkdown } from "./transform";

const grouped = `::: code-group

\`\`\`js [config.js]
export default {}
\`\`\`

\`\`\`ts [config.ts]
export default {}
\`\`\`

:::
`;

describe("code group transform", () => {
  it("expands labeled fences into the existing no-JS tab widget", async () => {
    resetTabGroupCounter();
    const enabled = await transformMarkdown(
      grouped,
      "docs/group.md",
      createDocsResolvedOptions({ highlight: false, codeGroups: { enabled: true } }),
    );
    expect(enabled.html).toContain("<tabs>");
    expect(enabled.html).toContain('label="config.js"');

    const widget = await transformAllPlugins(enabled.html, { github: false, openGraph: false });
    expect(widget).toContain("ox-tabs");
    expect(widget).toContain("<noscript>");
    expect(widget).toContain("config.js");
    expect(widget).toContain("config.ts");
    expect(widget).not.toContain("<script");

    const disabled = await transformMarkdown(
      grouped,
      "docs/group.md",
      createDocsResolvedOptions({ highlight: false, codeGroups: { enabled: false } }),
    );
    expect(disabled.html).not.toContain("<tabs>");
    expect(disabled.html).not.toContain("ox-tabs");
  });
});
