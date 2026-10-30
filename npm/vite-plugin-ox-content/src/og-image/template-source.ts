import type { OgImageTemplateFn } from "./types";

/** Compiled code includes imported components; Vue also contributes extracted CSS. */
export const templateSources = new WeakMap<OgImageTemplateFn, string>();
