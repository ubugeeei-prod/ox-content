import type { OxContentOptions, ResolvedImageGalleryOptions, ResolvedOptions } from "./types";

type JsImageGalleryOptions = {
  enabled: true;
  lazy?: boolean;
  missingAlt: "error" | "warn" | "ignore";
  empty: "error" | "warn" | "ignore";
};

const disabled: ResolvedImageGalleryOptions = {
  enabled: false,
  lazy: true,
  missingAlt: "error",
  empty: "error",
};

export function resolveImageGalleryOptions(
  options: OxContentOptions["imageGalleries"],
): ResolvedImageGalleryOptions {
  if (!options) return { ...disabled };
  if (options === true) {
    return { enabled: true, missingAlt: "error", empty: "error" };
  }
  if (options.enabled === false) return { ...disabled };
  return {
    enabled: options.enabled ?? true,
    lazy: options.lazy,
    missingAlt: reportMode(options.missingAlt),
    empty: reportMode(options.empty),
  };
}

export function toJsImageGalleryOptions(
  options: ResolvedOptions["imageGalleries"] | undefined,
  images: ResolvedOptions["images"] | undefined,
): JsImageGalleryOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    lazy: options.lazy ?? images?.lazy ?? true,
    missingAlt: options.missingAlt,
    empty: options.empty,
  };
}

function reportMode(value: unknown): "error" | "warn" | "ignore" {
  return value === "warn" || value === "ignore" ? value : "error";
}
