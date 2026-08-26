import type { OxContentOptions, ResolvedConditionalBlockOptions } from "./types";

type JsConditionalBlockOptions = {
  enabled: true;
  values: Record<string, unknown>;
};

const disabled: ResolvedConditionalBlockOptions = {
  enabled: false,
  values: {},
};

export function resolveConditionalBlockOptions(
  options: OxContentOptions["conditionalBlocks"],
): ResolvedConditionalBlockOptions {
  if (!options) return { ...disabled };
  if (options === true) return { enabled: true, values: {} };
  if (options.enabled === false) return { ...disabled };
  return {
    enabled: options.enabled ?? true,
    values: options.values ?? {},
  };
}

export function toJsConditionalBlockOptions(
  options: ResolvedConditionalBlockOptions | undefined,
): JsConditionalBlockOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    values: options.values,
  };
}
