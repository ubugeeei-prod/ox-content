import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import solid from "@solidjs/vite-plugin";
import { describe, expect, it } from "vite-plus/test";
import { build, type Plugin, type ResolvedConfig } from "vite";
import { oxContentSolid } from "./index";
import type { SolidIntegrationOptions } from "./types";
import { formatSolidPluginError } from "./verify";

describe("oxContentSolid", () => {
  const packageNodeModules = fileURLToPath(new URL("../node_modules/", import.meta.url));

  it("returns the transform, verify, environment and hmr plugins", () => {
    const names = pluginNames(oxContentSolid());

    expect(names).toContain("ox-content:solid-transform");
    expect(names).toContain("ox-content:solid-verify");
    expect(names).toContain("ox-content:solid-environment");
    expect(names).toContain("ox-content:solid-hmr");
  });

  it("accepts a config where @solidjs/vite-plugin runs after it", async () => {
    await expect(
      resolveConfigWith({}, ["ox-content:solid-transform", "solid"]),
    ).resolves.toBeUndefined();
  });

  it("rejects a config without @solidjs/vite-plugin", async () => {
    await expect(resolveConfigWith({}, ["ox-content:solid-transform"])).rejects.toThrow(
      /@solidjs\/vite-plugin was not found/,
    );
  });

  it("rejects a config where @solidjs/vite-plugin runs first", async () => {
    // Solid would receive raw Markdown instead of the generated JSX.
    await expect(resolveConfigWith({}, ["solid", "ox-content:solid-transform"])).rejects.toThrow(
      /runs before oxContentSolid\(\)/,
    );
  });

  it("skips the check when verifySolidPlugin is disabled", async () => {
    await expect(
      resolveConfigWith({ verifySolidPlugin: false }, ["ox-content:solid-transform"]),
    ).resolves.toBeUndefined();
  });

  it("reports an uncompiled module as a missing extensions entry", () => {
    const verify = findPlugin(oxContentSolid(), "ox-content:solid-verify");
    const errors: string[] = [];
    const context = {
      error(message: string) {
        errors.push(message);
        throw new Error(message);
      },
    };

    const runTransform = (code: string, id: string) =>
      (verify.transform as (this: unknown, code: string, id: string) => unknown).call(
        context,
        code,
        id,
      );

    expect(() =>
      runTransform('<div class="ox-content" innerHTML={rawHtml} />', "/docs/a.md"),
    ).toThrow(/`extensions` option must list \.md, \.markdown, and \.mdx/);

    // Compiled output and non-Markdown ids pass straight through.
    expect(runTransform("_$template(`<div class=ox-content>`)", "/docs/a.md")).toBeNull();
    expect(runTransform('<div class="ox-content" innerHTML={rawHtml} />', "/src/a.ts")).toBeNull();
  });

  it("formats setup diagnostics for the Solid 2 native compiler", () => {
    const message = formatSolidPluginError("extensions");

    expect(message).toContain("@solidjs/vite-plugin");
    expect(message).toContain("compiler: 'native'");
    expect(message).not.toContain("babel-preset-solid");
  });

  it("builds Markdown and MDX modules with Solid 2's native compiler", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-solid-native-"));

    try {
      await fs.mkdir(path.join(root, "docs", "widgets"), { recursive: true });
      await fs.mkdir(path.join(root, "src", "components"), { recursive: true });

      await fs.writeFile(
        path.join(root, "index.html"),
        '<div id="app"></div><script type="module" src="/src/main.tsx"></script>',
      );
      await fs.writeFile(
        path.join(root, "src", "main.tsx"),
        [
          'import { render } from "@solidjs/web";',
          'import StaticDoc from "../docs/static.md";',
          'import NotesDoc from "../docs/notes.markdown";',
          'import IslandDoc from "../docs/island.mdx";',
          "",
          'render(() => <><StaticDoc /><NotesDoc /><IslandDoc /></>, document.getElementById("app")!);',
        ].join("\n"),
      );
      await fs.writeFile(
        path.join(root, "src", "components", "Alert.tsx"),
        [
          'import type { JSX } from "solid-js";',
          "",
          "export default function Alert(props: { tone?: string; children?: JSX.Element }) {",
          '  return <section data-alert={props.tone ?? "info"}>{props.children}</section>;',
          "}",
        ].join("\n"),
      );
      await fs.writeFile(path.join(root, "docs", "static.md"), "# Static\n\nPlain Markdown.");
      await fs.writeFile(path.join(root, "docs", "notes.markdown"), "# Notes\n\nMore Markdown.");
      await fs.writeFile(
        path.join(root, "docs", "widgets", "Badge.tsx"),
        [
          "export default function Badge(props: { label: string }) {",
          "  return <span data-badge={props.label}>{props.label}</span>;",
          "}",
        ].join("\n"),
      );
      await fs.writeFile(
        path.join(root, "docs", "island.mdx"),
        [
          "import Badge from './widgets/Badge.tsx'",
          "",
          "# Island",
          "",
          '<Alert tone="success">Registered island</Alert>',
          "",
          '<Badge label="local" />',
        ].join("\n"),
      );

      await build({
        root,
        configFile: false,
        logLevel: "silent",
        plugins: [
          oxContentSolid({
            srcDir: "docs",
            components: { Alert: "./src/components/Alert.tsx" },
            embeds: { github: false, openGraph: false },
          }),
          solid({ extensions: [".md", ".markdown", ".mdx"], compiler: "native" }),
        ],
        resolve: {
          alias: {
            "@ox-content/islands": fileURLToPath(
              new URL("../../ox-content-islands/src/index.ts", import.meta.url),
            ),
            "@solidjs/web": path.join(packageNodeModules, "@solidjs/web/dist/web.js"),
            "solid-js": path.join(packageNodeModules, "solid-js/dist/solid.js"),
          },
        },
        build: {
          outDir: "dist",
          emptyOutDir: true,
          rollupOptions: {
            input: path.join(root, "index.html"),
          },
        },
      });

      const assetDir = path.join(root, "dist", "assets");
      const bundle = (
        await Promise.all(
          (await fs.readdir(assetDir))
            .filter((file) => file.endsWith(".js"))
            .map((file) => fs.readFile(path.join(assetDir, file), "utf8")),
        )
      ).join("\n");

      expect(bundle).not.toContain("innerHTML={rawHtml}");
      expect(bundle).not.toContain("<Alert");
      expect(bundle).not.toContain("<Badge");
      expect(bundle).toContain("data-ox-island");
    } finally {
      await fs.rm(root, { force: true, recursive: true });
    }
  });

  it("invalidates Markdown modules when a registered Solid component changes", () => {
    const hmr = findPlugin(
      oxContentSolid({ components: { Alert: "./src/components/Alert.tsx" } }),
      "ox-content:solid-hmr",
    );
    const changedModule = { file: "/repo/src/components/Alert.tsx" };
    const markdownModule = { file: "/repo/docs/index.md" };
    const messages: unknown[] = [];
    const context = {
      file: "/repo/src/components/Alert.tsx",
      modules: [changedModule],
      server: {
        moduleGraph: {
          idToModuleMap: new Map([["/repo/docs/index.md", markdownModule]]),
        },
        ws: {
          send(message: unknown) {
            messages.push(message);
          },
        },
      },
    };

    const result = (hmr.handleHotUpdate as (context: unknown) => unknown)(context);

    expect(result).toEqual([changedModule, markdownModule]);
    expect(messages).toEqual([
      {
        type: "custom",
        event: "ox-content:solid-update",
        data: { file: "/repo/src/components/Alert.tsx" },
      },
    ]);
  });
});

function pluginNames(plugins: ReturnType<typeof oxContentSolid>): string[] {
  return (plugins as Plugin[]).map((plugin) => plugin.name);
}

function findPlugin(plugins: ReturnType<typeof oxContentSolid>, name: string): Plugin {
  const plugin = (plugins as Plugin[]).find((candidate) => candidate.name === name);
  if (!plugin) throw new Error(`plugin ${name} not found`);
  return plugin;
}

/**
 * Drives `configResolved` on the transform plugin with a plugin list standing in
 * for a resolved Vite config. Only the plugin names matter to the check.
 */
async function resolveConfigWith(
  options: SolidIntegrationOptions,
  pluginNames: string[],
): Promise<void> {
  const transform = findPlugin(oxContentSolid(options), "ox-content:solid-transform");
  const config = {
    root: "/repo",
    plugins: pluginNames.map((name) => ({ name })),
  } as unknown as ResolvedConfig;

  const hook = transform.configResolved as (config: ResolvedConfig) => void | Promise<void>;
  await hook.call(transform, config);
}
