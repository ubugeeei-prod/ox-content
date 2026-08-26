import type { OxContentOptions, ResolvedOptions } from "./types";

type JsDataTableOptions = {
  enabled: true;
  rootDir?: string;
  missing: "error" | "warn";
};

const disabled: ResolvedOptions["dataTables"] = {
  enabled: false,
  missing: "error",
};

export function resolveDataTableOptions(
  options: OxContentOptions["dataTables"],
): ResolvedOptions["dataTables"] {
  if (!options) return { ...disabled };
  if (options === true) return { enabled: true, missing: "error" };
  return {
    enabled: options.enabled ?? true,
    rootDir: options.rootDir,
    missing: options.missing === "warn" ? "warn" : "error",
  };
}

export function toJsDataTableOptions(
  options: ResolvedOptions["dataTables"] | undefined,
): JsDataTableOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    rootDir: options.rootDir,
    missing: options.missing,
  };
}
