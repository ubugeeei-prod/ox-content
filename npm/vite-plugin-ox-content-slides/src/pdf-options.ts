/**
 * PDF export configuration for a generated slide deck.
 */
export interface SlidePdfOptions {
  enabled?: boolean;
  fileName?: string;
  pageWidth?: string;
  pageHeight?: string;
  scale?: number;
}

/**
 * Normalized PDF export options.
 */
export interface ResolvedSlidePdfOptions {
  enabled: boolean;
  fileName: string;
  pageWidth: string;
  pageHeight: string;
  scale: number;
}

const DEFAULT_FILE_NAME = "deck.pdf";
const DEFAULT_PAGE_WIDTH = "13.333in";
const DEFAULT_PAGE_HEIGHT = "7.5in";
const DEFAULT_SCALE = 1;

function normalizeScale(input: number | undefined): number {
  if (input === undefined || !Number.isFinite(input)) {
    return DEFAULT_SCALE;
  }

  return Math.min(2, Math.max(0.1, input));
}

/**
 * Resolves public PDF settings into runtime defaults.
 */
export function resolveSlidePdfOptions(
  options: SlidePdfOptions | boolean | undefined,
): ResolvedSlidePdfOptions {
  if (options === false || options === undefined) {
    return {
      enabled: false,
      fileName: DEFAULT_FILE_NAME,
      pageWidth: DEFAULT_PAGE_WIDTH,
      pageHeight: DEFAULT_PAGE_HEIGHT,
      scale: DEFAULT_SCALE,
    };
  }

  if (options === true) {
    return {
      enabled: true,
      fileName: DEFAULT_FILE_NAME,
      pageWidth: DEFAULT_PAGE_WIDTH,
      pageHeight: DEFAULT_PAGE_HEIGHT,
      scale: DEFAULT_SCALE,
    };
  }

  return {
    enabled: options.enabled ?? true,
    fileName: options.fileName?.trim() || DEFAULT_FILE_NAME,
    pageWidth: options.pageWidth?.trim() || DEFAULT_PAGE_WIDTH,
    pageHeight: options.pageHeight?.trim() || DEFAULT_PAGE_HEIGHT,
    scale: normalizeScale(options.scale),
  };
}
