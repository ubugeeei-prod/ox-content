export interface VideoMeta {
  title?: string;
  author?: string;
  image?: string;
  duration?: string;
  views?: string;
  status?: string;
}

export function vimeoMetaFromJson(value: unknown): VideoMeta | null {
  const item = record(value);
  if (!item) return null;
  return compactMeta({
    title: trimmed(item.title),
    author: trimmed(item.author_name),
    image: safeHttps(trimmed(item.thumbnail_url)),
    duration: durationLabel(item.duration),
  });
}

function safeHttps(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

function durationLabel(value: unknown): string | undefined {
  const seconds = typeof value === "number" && Number.isFinite(value) ? Math.floor(value) : null;
  if (!seconds || seconds < 1) return undefined;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = seconds % 60;
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${String(rest).padStart(2, "0")}`
    : `${minutes}:${String(rest).padStart(2, "0")}`;
}

function compactMeta(meta: VideoMeta): VideoMeta {
  return Object.fromEntries(Object.entries(meta).filter(([, value]) => value)) as VideoMeta;
}

function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
