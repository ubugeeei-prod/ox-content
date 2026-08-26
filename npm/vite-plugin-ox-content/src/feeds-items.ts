import { parseDate } from "./feed-format";
import type { FeedEntry } from "./feed-format";
import { homePageUrl } from "./feeds-output";
import { classifyPublishState } from "./publish-state";
import type { ResolvedPublishStateOptions } from "./types";
import type { FeedItemInput, FeedsRenderInput } from "./feeds";

const DEFAULT_LIMIT = 20;

function rawItems(input: FeedsRenderInput): readonly FeedItemInput[] {
  if (input.items) {
    return input.items;
  }
  const names = input.collectionNames ?? Object.keys(input.collections ?? {});
  const name = resolveFeedCollectionName(input.options?.collection, names);
  return name ? (input.collections?.[name] ?? []) : [];
}

function resolveFeedCollectionName(
  requested: string | undefined,
  collectionNames: readonly string[],
): string | undefined {
  if (requested) {
    return requested;
  }
  if (collectionNames.includes("content")) {
    return "content";
  }
  return collectionNames[0];
}

export function publishedItems(input: FeedsRenderInput): FeedEntry[] {
  const published = rawItems(input)
    .filter((item) => !isExcludedFromFeed(item, input.publishState))
    .map((item) => normalizeItem(item, input))
    .filter((item) => item.loc.length > 0);
  published.sort((left, right) => {
    const dateCmp =
      (right.date?.unix ?? Number.NEGATIVE_INFINITY) -
      (left.date?.unix ?? Number.NEGATIVE_INFINITY);
    return dateCmp !== 0 ? dateCmp : left.loc < right.loc ? -1 : left.loc > right.loc ? 1 : 0;
  });
  return published.slice(0, input.options?.limit ?? DEFAULT_LIMIT);
}

function isExcludedFromFeed(
  item: FeedItemInput,
  publishState: ResolvedPublishStateOptions | undefined,
): boolean {
  const frontmatter = item.frontmatter ?? {};
  if (item.draft === true || frontmatter.draft === true) {
    return true;
  }
  if (item.unlisted === true || frontmatter.unlisted === true) {
    return true;
  }
  if (frontmatter.external === true) {
    return true;
  }
  if (!publishState?.enabled) {
    return false;
  }
  return !classifyPublishState(
    {
      ...frontmatter,
      ...(item.draft === true ? { draft: true } : {}),
      ...(item.unlisted === true ? { unlisted: true } : {}),
    },
    publishState,
  ).listed;
}

function normalizeItem(item: FeedItemInput, input: FeedsRenderInput): FeedEntry {
  return {
    title: item.title ?? "",
    description: typeof item.description === "string" ? item.description : undefined,
    loc: item.loc || itemLoc(input, item),
    date:
      parseDate(dateField(item.date ?? item.frontmatter?.date)) ??
      parseDate(dateField(item.lastUpdated ?? item.frontmatter?.lastUpdated)),
  };
}

function itemLoc(input: FeedsRenderInput, item: FeedItemInput): string {
  const home = homePageUrl(input.siteUrl, input.base);
  const urlPath = (item.path ?? "").replace(/^\/+|\/+$/g, "");
  return urlPath ? `${home}${urlPath}/` : home;
}

function dateField(value: unknown): string | undefined {
  if (typeof value === "string" && value.trim()) {
    return value.trim();
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  if (value instanceof Date && !Number.isNaN(value.getTime())) {
    return value.toISOString();
  }
  return undefined;
}
