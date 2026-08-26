import { fetchRedditPostData } from "./fetch";
import { renderRedditErrorCard, renderRedditFallbackCard, renderRedditPostCard } from "./render";
import type { RedditEmbedOptions, RedditPostData, RedditPostReference } from "./types";
import { resolveRedditEmbedOptions } from "./types";
import { parseRedditPostId, parseRedditPostReference, redditReferenceKey } from "./url";

const REDDIT_ELEMENT = /<reddit\b((?:[^>"']|"[^"]*"|'[^']*')*)>[\s\S]*?<\/reddit\s*>/gi;

export function redditElementAttributes(attributes: string): {
  reference: RedditPostReference | null;
} {
  const values = new Map<string, string>();
  const pattern = /\b(url|href|permalink|id)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi;
  for (const match of attributes.matchAll(pattern)) {
    values.set(match[1].toLowerCase(), match[2] ?? match[3] ?? match[4] ?? "");
  }

  const url = values.get("url") ?? values.get("href") ?? values.get("permalink");
  return {
    reference: url ? parseRedditPostReference(url) : parseRedditPostId(values.get("id") ?? ""),
  };
}

export function collectRedditPostReferences(html: string): RedditPostReference[] {
  const references: RedditPostReference[] = [];
  for (const match of html.matchAll(REDDIT_ELEMENT)) {
    const reference = redditElementAttributes(match[1] ?? "").reference;
    if (reference) references.push(reference);
  }
  return references;
}

export async function prefetchRedditPosts(
  references: RedditPostReference[],
  options: RedditEmbedOptions = {},
): Promise<Map<string, RedditPostData | null>> {
  const resolved = resolveRedditEmbedOptions(options);
  const results = new Map<string, RedditPostData | null>();
  const unique = new Map<string, RedditPostReference>();

  for (const reference of references) {
    if (reference.apiUrl) unique.set(redditReferenceKey(reference), reference);
  }

  await Promise.all(
    [...unique.entries()].map(async ([key, reference]) => {
      results.set(key, await fetchRedditPostData(reference, resolved));
    }),
  );

  return results;
}

export async function transformRedditEmbeds(
  html: string,
  options: RedditEmbedOptions = {},
): Promise<string> {
  if (!/<reddit\b/i.test(html)) return html;

  const resolved = resolveRedditEmbedOptions(options);
  const data = resolved.fetch
    ? await prefetchRedditPosts(collectRedditPostReferences(html), options)
    : new Map<string, RedditPostData | null>();

  let output = "";
  let cursor = 0;
  for (const match of html.matchAll(REDDIT_ELEMENT)) {
    const index = match.index ?? 0;
    output += html.slice(cursor, index);

    const reference = redditElementAttributes(match[1] ?? "").reference;
    if (!reference) {
      output += renderRedditErrorCard();
    } else if (!resolved.fetch) {
      output += renderRedditFallbackCard(reference, "Metadata fetch disabled");
    } else {
      const post = data.get(redditReferenceKey(reference));
      output += post ? renderRedditPostCard(post) : renderRedditFallbackCard(reference);
    }

    cursor = index + match[0].length;
  }

  return output + html.slice(cursor);
}
