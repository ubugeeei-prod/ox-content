import type { OxContentOptions, ResolvedOptions, ResolvedTimelineOptions } from "./types";

type ReportMode = "error" | "warn" | "ignore";

type JsTimelineOptions = {
  enabled: true;
  ordered: boolean;
  invalidDate: ReportMode;
  unknownMeta: ReportMode;
  empty: ReportMode;
};

const disabled: ResolvedTimelineOptions = {
  enabled: false,
  ordered: true,
  invalidDate: "error",
  unknownMeta: "error",
  empty: "error",
};

export function resolveTimelineOptions(
  options: OxContentOptions["timelines"],
): ResolvedTimelineOptions {
  if (!options) return { ...disabled };
  if (options === true) {
    return {
      enabled: true,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    };
  }
  if (options.enabled === false) return { ...disabled };
  return {
    enabled: options.enabled ?? true,
    ordered: options.ordered ?? true,
    invalidDate: reportMode(options.invalidDate),
    unknownMeta: reportMode(options.unknownMeta),
    empty: reportMode(options.empty),
  };
}

export function toJsTimelineOptions(
  options: ResolvedOptions["timelines"] | undefined,
): JsTimelineOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    ordered: options.ordered,
    invalidDate: options.invalidDate,
    unknownMeta: options.unknownMeta,
    empty: options.empty,
  };
}

function reportMode(value: unknown): ReportMode {
  return value === "warn" || value === "ignore" ? value : "error";
}
