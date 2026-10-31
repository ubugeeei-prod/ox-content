import type { OxContentOptions } from "./types";

export function customHostOxContentOptions(options: OxContentOptions = {}): OxContentOptions {
  const ssg = options.ssg;
  const disabledSsg =
    ssg && typeof ssg === "object" ? { ...ssg, enabled: false } : { enabled: false };
  return { ...options, ssg: disabledSsg };
}
