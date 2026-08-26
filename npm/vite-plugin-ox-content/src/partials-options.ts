import type { OxContentOptions, ResolvedOptions } from "./types";

type JsPartialsOptions = {
  enabled: true;
  rootDir?: string;
  root: string;
  missing: "literal" | "error";
};

const disabled: ResolvedOptions["partials"] = {
  enabled: false,
  root: "_partials",
  missing: "literal",
};

export function resolvePartialsOptions(
  options: OxContentOptions["partials"],
): NonNullable<ResolvedOptions["partials"]> {
  if (!options) return { ...disabled };
  if (options === true) return { enabled: true, root: "_partials", missing: "literal" };
  return {
    enabled: options.enabled ?? true,
    rootDir: options.rootDir,
    root: options.root?.trim() || "_partials",
    missing: options.missing === "error" ? "error" : "literal",
  };
}

export function toJsPartialsOptions(
  options: ResolvedOptions["partials"] | undefined,
): JsPartialsOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    rootDir: options.rootDir,
    root: options.root,
    missing: options.missing,
  };
}
