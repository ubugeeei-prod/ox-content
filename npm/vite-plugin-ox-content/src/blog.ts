/**
 * Opt-in blog index, authors, tags, reading time, and archive.
 */

export type { BlogSourcePage } from "./blog-html";
export { resolveBlogOptions, resolveBlogCollectionName } from "./blog-options";
export {
  BlogFeedError,
  loadBlogFeedEntries,
  loadExternalBlogPosts,
  mergeBlogFeedEntries,
  mergeBlogPosts,
  type BlogFeedEntry,
  type BlogFeedFetchFn,
  type BlogFeedFetchLimits,
  type BlogFeedLookup,
  type BlogFeedNetwork,
  type LoadBlogFeedEntriesInput,
  type LoadBlogFeedEntriesResult,
} from "./blog-feeds";
export { readingTimeMinutes } from "./blog-reading";
export {
  appendBlogPages,
  injectBlogPostMeta,
  toBlogProcessResult,
  type BlogGeneratedPage,
} from "./blog-pages";
