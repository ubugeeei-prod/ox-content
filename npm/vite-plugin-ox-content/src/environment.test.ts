import { describe, expect, it } from "vite-plus/test";
import {
  createMarkdownEnvironment,
  createRuntimeResolveConditions,
  mergeResolveConditions,
} from "./environment";
import { oxContent } from "./index";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

describe("markdown environment", () => {
  it("adds runtime-specific conditions for non-Node Vite hosts", () => {
    const deno = createMarkdownEnvironment(createDocsResolvedOptions(), "deno");
    const bun = createMarkdownEnvironment(createDocsResolvedOptions(), "bun");

    expect(deno.resolve?.conditions).toEqual(["markdown", "deno", "node", "import"]);
    expect(bun.resolve?.conditions).toEqual(["markdown", "bun", "node", "import"]);
    expect(deno.build?.target).toBe("esnext");
    expect(bun.build?.target).toBe("esnext");
  });

  it("keeps the Node environment condition list stable", () => {
    const env = createMarkdownEnvironment(createDocsResolvedOptions(), "node");

    expect(env.resolve?.conditions).toEqual(["markdown", "node", "import"]);
    expect(env.build?.target).toBe("node18");
    expect(createRuntimeResolveConditions("node")).toEqual([]);
  });

  it("deduplicates environment conditions without reordering existing ones", () => {
    expect(mergeResolveConditions(["custom", "node"], ["markdown", "node", "import"])).toEqual([
      "custom",
      "node",
      "markdown",
      "import",
    ]);
  });

  it("configures the markdown environment without dropping user conditions", async () => {
    const plugin = oxContent({}).find((candidate) => candidate.name === "ox-content:environment");
    if (!plugin?.configEnvironment) {
      throw new Error("environment plugin should expose configEnvironment");
    }

    const result = await (
      plugin.configEnvironment as (
        name: string,
        config: { resolve?: { conditions?: string[] } },
      ) => unknown
    )("markdown", { resolve: { conditions: ["custom", "node"] } });

    expect(result).toEqual({
      resolve: {
        conditions: ["custom", "node", "markdown", "import"],
      },
    });
  });

  it("sends Markdown HMR through the current Vite environment", () => {
    const plugin = oxContent({}).find((candidate) => candidate.name === "ox-content");
    if (!plugin?.hotUpdate) {
      throw new Error("main plugin should expose hotUpdate");
    }
    const messages: unknown[] = [];
    const modules = [{ id: "/content/guide.md" }];

    const result = (
      plugin.hotUpdate as (this: unknown, input: { file: string; modules: unknown[] }) => unknown
    ).call(
      {
        environment: {
          hot: {
            send(message: unknown) {
              messages.push(message);
            },
          },
        },
      },
      { file: "/repo/content/guide.md", modules },
    );

    expect(result).toBe(modules);
    expect(messages).toEqual([
      {
        type: "custom",
        event: "ox-content:update",
        data: { file: "/repo/content/guide.md" },
      },
    ]);
  });
});
