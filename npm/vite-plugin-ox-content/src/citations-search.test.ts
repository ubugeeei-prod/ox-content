import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { collectCitationSearchText, resolveCitationsOptions } from "./citations";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("citation search text", () => {
  it("does not read a linked scoped package name as a citation", async () => {
    // The search-index pass reads Markdown source, where a link label opening
    // with `@` is indistinguishable from a citation group by that `@` alone.
    // These failed the whole index with `malformed citation`, because a scoped
    // package name carries a `/` that no citation key may contain.
    const markdown = [
      "- [@ox-content/vite-plugin](./packages/vite-plugin-ox-content.md)",
      "- [@ox-content/napi](./packages/napi.md)",
      "- [@ox-content/wasm](./packages/wasm.md)",
      "- [@ox-content/napi][napi-ref]",
      "",
      "[napi-ref]: ./packages/napi.md",
    ].join("\n");

    await expect(collectCitationSearchText(markdown, await options([]))).resolves.toBe("");
  });

  it("still collects a real citation alongside linked package names", async () => {
    const markdown = ["See [@ox-content/napi](./packages/napi.md) and the spec [@rfc9110]."].join(
      "\n",
    );

    const text = await collectCitationSearchText(
      markdown,
      await options([{ id: "rfc9110", title: "HTTP Semantics" }]),
    );

    expect(text).toContain("HTTP Semantics");
  });

  it("still reports a citation group that is genuinely malformed", async () => {
    await expect(collectCitationSearchText("Bad [@bad key].", await options([]))).rejects.toThrow(
      'malformed citation "[@bad key]"',
    );
  });
});

async function options(items: Array<Record<string, unknown>>) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-citation-search-"));
  tempDirs.push(root);
  await fs.writeFile(path.join(root, "refs.json"), JSON.stringify(items), "utf8");
  return resolveCitationsOptions({ bibliography: ["refs.json"], rootDir: root });
}
