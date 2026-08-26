import * as path from "node:path";
import type { FeedFormat, ResolvedFeedChannel } from "./types";

const MISSING_SITE_URL =
  "[ox-content] feeds is enabled but ssg.siteUrl is not set; RSS, Atom, and JSON feeds were not written";
const UNSAFE_SITE_URL =
  "[ox-content] feeds requires ssg.siteUrl to be a safe absolute http(s) URL; RSS, Atom, and JSON feeds were not written";

const FORMAT_OUTPUTS: Record<FeedFormat, string> = {
  rss: "feed.xml",
  atom: "atom.xml",
  json: "feed.json",
};

export function homePageUrl(siteUrl: string | undefined, base = "/"): string {
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return `${origin}${prefix}`;
}

export function siteUrlWarning(siteUrl: string | undefined): string | undefined {
  const trimmed = siteUrl?.trim();
  if (!trimmed) {
    return MISSING_SITE_URL;
  }
  return isSafeHttpUrl(trimmed) ? undefined : UNSAFE_SITE_URL;
}

function isSafeHttpUrl(value: string): boolean {
  if (/\s/u.test(value)) {
    return false;
  }
  const lower = value.toLowerCase();
  if (!lower.startsWith("https://") && !lower.startsWith("http://")) {
    return false;
  }
  try {
    const url = new URL(value);
    return (url.protocol === "https:" || url.protocol === "http:") && url.hostname.length > 0;
  } catch {
    return false;
  }
}

export function outputDir(outDir: string, feedPath: string): string | undefined {
  const relative = feedPath.replace(/^\/+|\/+$/g, "");
  if (!relative) {
    return outDir;
  }
  if (relative.includes("\\")) {
    return undefined;
  }
  const normalized = path.posix.normalize(relative);
  if (normalized === ".." || normalized.startsWith("../")) {
    return undefined;
  }
  return path.join(outDir, ...normalized.split("/"));
}

export function feedOutputWarning(
  outDir: string,
  channels: readonly ResolvedFeedChannel[],
): string | undefined {
  const seen = new Map<string, string>();
  for (const [index, channel] of channels.entries()) {
    const dest = outputDir(outDir, channel.path);
    if (!dest) {
      return unsafeFeedPathWarning(channel, index);
    }
    for (const fileName of feedOutputFileNames(channel.formats)) {
      const outputPath = path.join(dest, fileName);
      const previous = seen.get(outputPath);
      const label = feedChannelLabel(channel, index);
      if (previous) {
        return `[ox-content] feeds output path "${displayOutputPath(outDir, outputPath)}" is used by both ${previous} and ${label}; feed files were not written`;
      }
      seen.set(outputPath, label);
    }
  }
  return undefined;
}

function feedOutputFileNames(formats: readonly FeedFormat[]): string[] {
  const seen = new Set<string>();
  for (const format of formats) {
    const fileName = FORMAT_OUTPUTS[format];
    if (fileName) {
      seen.add(fileName);
    }
  }
  return [...seen];
}

function feedChannelLabel(channel: ResolvedFeedChannel, index: number): string {
  return JSON.stringify(channel.name ?? channel.collection ?? `#${index + 1}`);
}

export function unsafeFeedPathWarning(channel: ResolvedFeedChannel, index: number): string {
  return `[ox-content] feeds channel ${feedChannelLabel(channel, index)} uses an unsafe output path; feed files were not written`;
}

function displayOutputPath(outDir: string, outputPath: string): string {
  return path.relative(outDir, outputPath).split(path.sep).join("/");
}
