import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { createSyndicationToken } from "./url";
import type { ResolvedTwitterEmbedOptions, TweetAssets, TweetData, TweetMedia } from "./types";

const tweetCache = new Map<string, TweetData>();

export function clearTweetCache(): void {
  tweetCache.clear();
}

export async function fetchTweetData(
  id: string,
  options: ResolvedTwitterEmbedOptions,
): Promise<TweetData | null> {
  const key = `${id}-${sanitizeSegment(options.lang)}`;
  if (options.cache) {
    const memory = tweetCache.get(key);
    if (memory) return memory;
    const disk = await readCachedTweet(key, options.cacheDir);
    if (disk) {
      tweetCache.set(key, disk);
      return disk;
    }
  }

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  const endpoint = new URL("https://cdn.syndication.twimg.com/tweet-result");
  endpoint.searchParams.set("id", id);
  endpoint.searchParams.set("lang", options.lang);
  endpoint.searchParams.set("token", createSyndicationToken(id));

  try {
    const response = await fetch(endpoint, {
      headers: { Accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) return null;
    const data: unknown = await response.json();
    if (!isTweetData(data)) return null;
    if (options.cache) {
      tweetCache.set(key, data);
      await writeCachedTweet(key, data, options.cacheDir);
    }
    return data;
  } catch {
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

export async function materializeTweetAssets(
  id: string,
  data: TweetData,
  options: ResolvedTwitterEmbedOptions,
): Promise<TweetAssets> {
  const assets: TweetAssets = { media: [] };
  const avatarUrl = data.user.profile_image_url_https?.replace(/_normal(?=\.[^.]+$)/, "_bigger");
  if (avatarUrl) {
    assets.avatar = await downloadAsset(avatarUrl, `${id}-avatar`, options);
  }

  const media = data.mediaDetails ?? data.entities?.media ?? [];
  for (const [index, item] of media.entries()) {
    if (item.type && item.type !== "photo") continue;
    if (!item.media_url_https) continue;
    const src = await downloadAsset(item.media_url_https, `${id}-media-${index + 1}`, options);
    if (src) assets.media.push(assetRecord(src, item));
  }
  return assets;
}

async function downloadAsset(
  source: string,
  basename: string,
  options: ResolvedTwitterEmbedOptions,
): Promise<string | undefined> {
  let url: URL;
  try {
    url = new URL(source);
  } catch {
    return undefined;
  }
  if (url.protocol !== "https:" || url.hostname.toLowerCase() !== "pbs.twimg.com") {
    return undefined;
  }

  const extension = extensionFromUrl(url);
  const filename = `${basename}${extension}`;
  const output = path.join(options.mediaOutputDir, filename);
  try {
    await access(output);
    return joinPublicPath(options.mediaPublicPath, filename);
  } catch {
    // Download the missing asset below.
  }

  try {
    const response = await fetch(url, { headers: { Accept: "image/*" } });
    if (!response.ok) return undefined;
    await mkdir(options.mediaOutputDir, { recursive: true });
    await writeFile(output, new Uint8Array(await response.arrayBuffer()));
    return joinPublicPath(options.mediaPublicPath, filename);
  } catch {
    return undefined;
  }
}

async function readCachedTweet(key: string, directory: string): Promise<TweetData | null> {
  try {
    const data: unknown = JSON.parse(await readFile(path.join(directory, `${key}.json`), "utf8"));
    return isTweetData(data) ? data : null;
  } catch {
    return null;
  }
}

async function writeCachedTweet(key: string, data: TweetData, directory: string): Promise<void> {
  try {
    await mkdir(directory, { recursive: true });
    await writeFile(path.join(directory, `${key}.json`), `${JSON.stringify(data)}\n`);
  } catch {
    // A read-only cache directory must not fail the build.
  }
}

function isTweetData(data: unknown): data is TweetData {
  if (!data || typeof data !== "object") return false;
  const value = data as Partial<TweetData>;
  return (
    typeof value.text === "string" &&
    Boolean(value.user) &&
    typeof value.user?.name === "string" &&
    typeof value.user.screen_name === "string"
  );
}

function extensionFromUrl(url: URL): string {
  const match = url.pathname.match(/\.(jpe?g|png|webp|gif)$/i);
  return match ? `.${match[1].toLowerCase().replace("jpeg", "jpg")}` : ".jpg";
}

function joinPublicPath(prefix: string, filename: string): string {
  return `${prefix.replace(/\/$/, "")}/${filename}`;
}

function sanitizeSegment(value: string): string {
  return value.replaceAll(/[^a-zA-Z0-9_-]/g, "-");
}

function assetRecord(src: string, media: TweetMedia): TweetAssets["media"][number] {
  return {
    src,
    alt: media.ext_alt_text,
    width: media.original_info?.width,
    height: media.original_info?.height,
  };
}
