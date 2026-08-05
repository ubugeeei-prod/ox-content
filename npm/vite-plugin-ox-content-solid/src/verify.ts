/**
 * Guards against the two ways `vite-plugin-solid` can fail to compile the
 * Markdown modules this plugin emits.
 *
 * Both are configuration mistakes with silent-until-cryptic failure modes, so
 * each is checked as early as it can be observed: plugin presence and ordering
 * are known once the config resolves, while the `extensions` option is held in
 * the Solid plugin's closure and can only be inferred from whether the generated
 * JSX actually got compiled.
 */

import type { ResolvedConfig } from "vite";

export const TRANSFORM_PLUGIN_NAME = "ox-content:solid-transform";

const SOLID_PLUGIN_NAME = "solid";

/**
 * Emitted by both generated module shapes. Solid's JSX has no runtime factory,
 * so this attribute only survives into the final module when nothing compiled
 * the JSX away — which is exactly the misconfiguration worth reporting.
 */
export const UNCOMPILED_JSX_MARKER = "innerHTML={rawHtml}";

export function formatSolidPluginError(reason: "missing" | "ordering" | "extensions"): string {
  const example = [
    "  plugins: [",
    "    oxContentSolid({ srcDir: 'docs' }),",
    "    solid({ extensions: ['.md', '.markdown', '.mdx'] }),",
    "  ]",
  ].join("\n");

  const detail = {
    missing:
      "vite-plugin-solid was not found in the Vite config. Markdown files are emitted as Solid JSX, which only runs after babel-preset-solid compiles it.",
    ordering:
      "vite-plugin-solid runs before oxContentSolid(), so it sees raw Markdown instead of the generated JSX. Both plugins are `enforce: 'pre'`, so their order follows the `plugins` array.",
    extensions:
      "vite-plugin-solid did not compile the generated Markdown module. Its `extensions` option must list the Markdown extensions; by default it only looks at .jsx/.tsx files.",
  }[reason];

  return `[ox-content:solid] ${detail}\n\n${example}\n`;
}

/**
 * Throws when `vite-plugin-solid` is absent, or placed where it would see raw
 * Markdown instead of the JSX this plugin generates.
 */
export function verifySolidPluginOrder(config: ResolvedConfig): void {
  const names = config.plugins.map((plugin) => plugin.name);
  const solidIndex = names.indexOf(SOLID_PLUGIN_NAME);

  if (solidIndex === -1) {
    throw new Error(formatSolidPluginError("missing"));
  }

  const transformIndex = names.indexOf(TRANSFORM_PLUGIN_NAME);
  if (transformIndex !== -1 && solidIndex < transformIndex) {
    throw new Error(formatSolidPluginError("ordering"));
  }
}
