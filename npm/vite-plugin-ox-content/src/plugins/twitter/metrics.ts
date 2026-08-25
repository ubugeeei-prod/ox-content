import { escapeHtml } from "./html";
import type { TweetData } from "./types";

interface MetricSpec {
  value: unknown;
  singular: string;
  plural: string;
}

export interface TweetMetricOptions {
  replies?: boolean;
  reposts?: boolean;
  quotes?: boolean;
  likes?: boolean;
  views?: boolean;
}

export function renderTweetMetrics(data: TweetData, options: TweetMetricOptions = {}): string {
  const metrics = tweetMetrics(data, options).map(renderMetric).join("");
  return metrics ? `<div class="ox-tweet__metrics">${metrics}</div>` : "";
}

export function formatCount(value: unknown): string | undefined {
  const n = normalizeCount(value);
  if (n === undefined) return undefined;
  if (n > 999_999) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n > 999) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

function tweetMetrics(data: TweetData, options: TweetMetricOptions): MetricSpec[] {
  return [
    enabled(options.replies) && metric(data.conversation_count, "reply", "replies"),
    enabled(options.reposts) &&
      metric(first(data.repost_count, data.retweet_count), "repost", "reposts"),
    enabled(options.quotes) && metric(data.quote_count, "quote", "quotes"),
    enabled(options.likes) && metric(data.favorite_count, "like", "likes"),
    enabled(options.views) &&
      metric(first(data.view_count, data.views_count, data.impression_count), "view", "views"),
  ].filter((item): item is MetricSpec => Boolean(item));
}

function enabled(value: boolean | undefined): boolean {
  return value !== false;
}

function metric(value: unknown, singular: string, plural: string): MetricSpec | undefined {
  return normalizeCount(value) === undefined ? undefined : { value, singular, plural };
}

function renderMetric({ value, singular, plural }: MetricSpec): string {
  const count = normalizeCount(value);
  const formatted = formatCount(value);
  if (count === undefined || !formatted) return "";
  const label = count === 1 ? singular : plural;
  return `<span class="ox-tweet__metric"><strong>${escapeHtml(formatted)}</strong> ${label}</span>`;
}

function first(...values: unknown[]): unknown {
  return values.find((value) => normalizeCount(value) !== undefined);
}

function normalizeCount(value: unknown): number | undefined {
  if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
    return Math.floor(value);
  }
  if (typeof value === "string" && /^\d+$/.test(value)) {
    return Number(value);
  }
  return undefined;
}
