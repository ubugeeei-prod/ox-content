import path from "node:path";
import { fetchTweetData, materializeTweetAssets } from "./fetch";
import { renderFetchedTweet } from "./render";
import type { ResolvedTwitterEmbedOptions, TwitterEmbedOptions } from "./types";
import { referenceFromAttributes } from "./url";

const TWEET_ELEMENT = /<(tweet|xpost)\b([^>]*?)(?:\/\s*>|>[\s\S]*?<\/\1\s*>)/gi;

export function resolveTwitterEmbedOptions(
  options: TwitterEmbedOptions,
): ResolvedTwitterEmbedOptions {
  return {
    fetch: options.fetch ?? false,
    lang: options.lang ?? "en",
    timeout: options.timeout ?? 10000,
    cache: options.cache ?? true,
    cacheDir: path.resolve(options.cacheDir ?? ".cache/ox-content/twitter"),
    mediaOutputDir: path.resolve(options.mediaOutputDir ?? "public/ox-content/twitter"),
    mediaPublicPath: options.mediaPublicPath ?? "/ox-content/twitter",
  };
}

export async function transformFetchedTweets(
  html: string,
  options: TwitterEmbedOptions,
): Promise<string> {
  const resolved = resolveTwitterEmbedOptions(options);
  if (!resolved.fetch) return html;

  let output = "";
  let cursor = 0;
  for (const match of html.matchAll(TWEET_ELEMENT)) {
    const index = match.index ?? 0;
    output += html.slice(cursor, index);
    const reference = referenceFromAttributes(match[2]);
    if (!reference) {
      output += match[0];
      cursor = index + match[0].length;
      continue;
    }

    const data = await fetchTweetData(reference.id, resolved);
    if (!data) {
      output += match[0];
      cursor = index + match[0].length;
      continue;
    }

    const assets = await materializeTweetAssets(reference.id, data, resolved);
    output += renderFetchedTweet(reference.url, data, assets, resolved);
    cursor = index + match[0].length;
  }
  return output + html.slice(cursor);
}
