import { access, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { ResolvedTwitterEmbedOptions, TweetVideoVariant } from "./types";

const VIDEO_HOSTS = new Set(["pbs.twimg.com", "video.twimg.com"]);

export function selectBestMp4Url(
  variants: readonly TweetVideoVariant[] | undefined,
): string | undefined {
  const candidates = (variants ?? []).filter(
    (variant): variant is TweetVideoVariant & { url: string } =>
      isVideoMp4Type(variant.content_type) && isAllowedVideoUrl(variant.url),
  );
  if (candidates.length === 0) return undefined;

  return candidates.reduce((best, variant) => {
    const bestBitrate = best.bitrate ?? Number.NEGATIVE_INFINITY;
    const nextBitrate = variant.bitrate ?? Number.NEGATIVE_INFINITY;
    if (nextBitrate > bestBitrate) return variant;
    if (nextBitrate === bestBitrate && variant.url < best.url) return variant;
    return best;
  }).url;
}

export function isVideoMp4Type(value: string | null | undefined): boolean {
  return (value ?? "").split(";", 1)[0].trim().toLowerCase() === "video/mp4";
}

export function isAllowedVideoUrl(value: string | null | undefined): value is string {
  if (!value) return false;
  try {
    const url = new URL(value);
    return url.protocol === "https:" && VIDEO_HOSTS.has(url.hostname.toLowerCase());
  } catch {
    return false;
  }
}

export async function downloadVideoAsset(
  source: string,
  basename: string,
  options: ResolvedTwitterEmbedOptions,
): Promise<string | undefined> {
  if (!isAllowedVideoUrl(source)) return undefined;

  const filename = `${sanitizeFilename(basename)}.mp4`;
  const output = path.join(options.mediaOutputDir, filename);
  const publicPath = joinPublicPath(options.mediaPublicPath, filename);
  try {
    await access(output);
    return publicPath;
  } catch {
    // Download the missing asset below.
  }

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetch(source, {
      headers: { Accept: "video/mp4" },
      signal: controller.signal,
    });
    if (!response.ok) return undefined;
    if (!isVideoMp4Type(response.headers?.get("content-type"))) return undefined;

    const declared = Number(response.headers?.get("content-length"));
    if (Number.isFinite(declared) && declared > options.maxVideoBytes) return undefined;

    const bytes = new Uint8Array(await response.arrayBuffer());
    if (bytes.byteLength > options.maxVideoBytes) return undefined;

    await mkdir(options.mediaOutputDir, { recursive: true });
    await writeFile(output, bytes);
    return publicPath;
  } catch {
    return undefined;
  } finally {
    clearTimeout(timeout);
  }
}

function sanitizeFilename(value: string): string {
  return value.replaceAll(/[^a-zA-Z0-9_-]/g, "-") || "video";
}

function joinPublicPath(prefix: string, filename: string): string {
  return `${prefix.replace(/\/$/, "")}/${filename}`;
}
