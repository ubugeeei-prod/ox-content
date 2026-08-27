/**
 * Shapes the feed pipeline passes between item selection and the native
 * renderer. The bodies themselves come from `ox_content_ssg::generate_feeds`
 * via `feeds-native.ts`.
 */

import type { FeedItemAttachment, FeedItemAuthor } from "./types";

export interface ParsedDate {
  unix: number;
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
}

export interface FeedDocument {
  siteName: string;
  siteDescription?: string;
  home: string;
  atomUrl: string;
  jsonUrl: string;
}

export interface FeedEntry {
  title: string;
  description?: string;
  content?: string;
  loc: string;
  id?: string;
  date?: ParsedDate;
  authors?: FeedItemAuthor[];
  image?: string;
  attachments?: FeedItemAttachment[];
  language?: string;
}
