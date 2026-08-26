import { parseDate } from "./feed-date";
import type { FeedEntry } from "./feed-format";
import { homePageUrl } from "./feeds-output";
import { classifyPublishState } from "./publish-state";
import type {
  FeedItemAttachment,
  FeedItemAuthor,
  FeedItemInput,
  ResolvedPublishStateOptions,
} from "./types";
import type { FeedsRenderInput } from "./feeds";

const DEFAULT_LIMIT = 20;

function rawItems(input: FeedsRenderInput): readonly FeedItemInput[] {
  if (input.items) {
    return input.items;
  }
  if (Array.isArray(input.options?.items)) {
    return input.options.items;
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
  const description = stringField(item.description ?? item.frontmatter?.description);
  const content = stringField(item.content ?? item.frontmatter?.content);
  const loc = stringField(item.loc) ?? stringField(item.url) ?? itemLoc(input, item);
  const id = usesProgrammaticItems(input) ? stringField(item.id) : undefined;
  return {
    title: stringField(item.title) ?? "",
    description,
    content,
    loc,
    id: id ?? loc,
    date:
      parseDate(dateField(item.date ?? item.frontmatter?.date)) ??
      parseDate(dateField(item.lastUpdated ?? item.frontmatter?.lastUpdated)),
    authors: authorsField(item),
    image: stringField(item.image ?? item.frontmatter?.image),
    attachments: attachmentsField(item.attachments ?? item.frontmatter?.attachments),
    language: stringField(item.language ?? item.frontmatter?.language),
  };
}

function usesProgrammaticItems(input: FeedsRenderInput): boolean {
  return input.items !== undefined || Array.isArray(input.options?.items);
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

function stringField(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function authorsField(item: FeedItemInput): FeedItemAuthor[] | undefined {
  const values: unknown[] = [];
  values.push(item.author ?? item.frontmatter?.author);
  const authors = item.authors ?? item.frontmatter?.authors;
  if (Array.isArray(authors)) {
    values.push(...authors);
  } else {
    values.push(authors);
  }
  const normalized = values.flatMap((value) => {
    const author = normalizeAuthor(value);
    return author ? [author] : [];
  });
  return normalized.length ? normalized : undefined;
}

function normalizeAuthor(value: unknown): FeedItemAuthor | undefined {
  const name = typeof value === "string" ? value : isRecord(value) ? value.name : undefined;
  if (typeof name !== "string" || !name.trim()) {
    return undefined;
  }
  const url = isRecord(value) ? stringField(value.url) : undefined;
  return { name: name.trim(), ...(url ? { url } : {}) };
}

function attachmentsField(value: unknown): FeedItemAttachment[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const attachments = value.flatMap((item) => {
    if (!isRecord(item)) {
      return [];
    }
    const url = stringField(item.url);
    if (!url) {
      return [];
    }
    return [
      {
        url,
        ...copyString(item, "mimeType"),
        ...copyString(item, "title"),
        ...copyNumber(item, "sizeInBytes"),
        ...copyNumber(item, "durationInSeconds"),
      },
    ];
  });
  return attachments.length ? attachments : undefined;
}

function copyString(record: Record<string, unknown>, key: string): Record<string, string> {
  const value = stringField(record[key]);
  return value ? { [key]: value } : {};
}

function copyNumber(record: Record<string, unknown>, key: string): Record<string, number> {
  const value = record[key];
  return typeof value === "number" && Number.isFinite(value) ? { [key]: value } : {};
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value != null && !Array.isArray(value);
}
