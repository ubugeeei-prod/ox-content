import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { render } from "../bin/mermaid-render.mjs";

const executablePath = [
  process.env.PUPPETEER_EXECUTABLE_PATH,
  "/usr/bin/google-chrome",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
].find((path) => path && existsSync(path));
const browser = { executablePath, args: ["--no-sandbox", "--disable-setuid-sandbox"] };
afterEach(() => vi.unstubAllEnvs());

describe("bundled Mermaid renderer", () => {
  it.each([
    "flowchart TD\n  Start --> Finish",
    "sequenceDiagram\n  Alice->>Bob: Hello",
    "classDiagram\n  Animal <|-- Duck",
  ])(
    "renders standard diagrams: %s",
    async (source) => {
      const svg = await render(source, { browser });
      expect(svg).toContain("<svg");
      expect(svg).toContain("</svg>");
      expect(svg).not.toContain("<script");
    },
    20000,
  );

  it("returns parsing errors and closes the browser", async () => {
    await expect(render("this is not a diagram", { browser })).rejects.toThrow();
  }, 20000);

  it("works through native extraction and the renderer CLI", () => {
    if (executablePath) vi.stubEnv("PUPPETEER_EXECUTABLE_PATH", executablePath);
    const require = createRequire(import.meta.url);
    const napi = require("@ox-content/napi");
    const renderer = fileURLToPath(new URL("../bin/mermaid-render.mjs", import.meta.url));
    const result = napi.transformMermaid(
      '<pre><code class="language-mermaid">flowchart TD\nA --&gt; B</code></pre>',
      renderer,
    );
    expect(result.errors).toEqual([]);
    expect(result.html).toContain('class="ox-mermaid"');
    expect(result.html).toContain("<svg");
  }, 20000);
});
