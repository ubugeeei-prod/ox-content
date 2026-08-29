import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const distDir = join(packageRoot, "dist");
const runtimeSource = join(
  packageRoot,
  "../../crates/ox_content_ssg/src/html/theme_transition_runtime.js",
);

const runtime = await readFile(runtimeSource, "utf8");
const banner =
  "// Generated from crates/ox_content_ssg/src/html/theme_transition_runtime.js.\n" +
  "// Run npm/vite-plugin-ox-content/scripts/build-theme-transition-client.mjs.\n\n";
const declarations = `export interface ThemeTransitionOptions {
  /** Activation event. Supplies the reveal origin; omit for the viewport centre. */
  event?: Event;
  /** Theme being switched to. \`"light"\` collapses the circle, anything else grows it. */
  nextTheme?: string;
  /** Synchronous theme mutation, run while both snapshots are captured. */
  apply: () => void;
  /** Reveal duration in milliseconds. Defaults to 420. */
  duration?: number;
  /** Reveal easing. Defaults to \`"ease-in-out"\`. */
  easing?: string;
}

/**
 * Runs \`apply\` inside a circular view transition, or immediately when View
 * Transitions are unavailable or the reader asked for reduced motion.
 *
 * Resolves once the transition has settled. A skipped transition resolves too,
 * so the returned promise never rejects.
 */
export declare function applyThemeTransition(options: ThemeTransitionOptions): Promise<void>;
`;

await mkdir(distDir, { recursive: true });
await writeFile(
  join(distDir, "theme-transition-client.mjs"),
  `${banner}${runtime}\nexport { applyThemeTransition };\n`,
);
await writeFile(
  join(distDir, "theme-transition-client.cjs"),
  `${banner}"use strict";\n${runtime}\nmodule.exports = { applyThemeTransition };\n`,
);
await writeFile(join(distDir, "theme-transition-client.d.mts"), declarations);
await writeFile(join(distDir, "theme-transition-client.d.cts"), declarations);
