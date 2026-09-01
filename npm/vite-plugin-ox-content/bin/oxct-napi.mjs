import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export function loadNapi() {
  try {
    return require("@ox-content/napi");
  } catch (error) {
    throw new Error(
      `Failed to load @ox-content/napi for oxct. Build the native package with \`vp run build:napi\` when running from the repository.\n${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
