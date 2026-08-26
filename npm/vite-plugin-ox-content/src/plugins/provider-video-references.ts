import { decodeProviderArticleAttr } from "./provider-article-attrs";

const TWITCH_RESERVED_CHANNELS = new Set([
  "directory",
  "downloads",
  "jobs",
  "p",
  "settings",
  "subscriptions",
  "turbo",
  "videos",
]);

export interface VideoProviderReference {
  provider: "vimeo" | "twitch";
  kind: "video" | "clip" | "channel";
  canonicalUrl: string;
  title: string;
  author?: string;
  apiUrl?: string;
  vimeoId?: string;
  twitchVideoId?: string;
  twitchClip?: string;
  twitchChannel?: string;
}

export function parseVideoProviderReference(
  tag: string,
  input: string,
): VideoProviderReference | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.username || url.password) return null;
    const segments = safeSegments(url);
    if (!segments) return null;
    switch (tag.toLowerCase()) {
      case "vimeo":
        return vimeoReference(url, segments);
      case "twitch":
        return twitchReference(url, segments);
      default:
        return null;
    }
  } catch {
    return null;
  }
}

export function videoEmbedUrl(
  reference: VideoProviderReference,
  optionParents: string[],
  attrs: string,
): string | undefined {
  if (reference.provider === "vimeo" && reference.vimeoId) {
    return `https://player.vimeo.com/video/${reference.vimeoId}?dnt=1`;
  }
  if (reference.provider !== "twitch") return undefined;
  const attrParents = normalizeTwitchParents(
    readAttr(attrs, "parent") ?? readAttr(attrs, "embedParent"),
  );
  const parents = attrParents.length > 0 ? attrParents : optionParents;
  if (parents.length === 0) return undefined;
  const params = new URLSearchParams();
  if (reference.twitchVideoId) params.set("video", `v${reference.twitchVideoId}`);
  if (reference.twitchClip) params.set("clip", reference.twitchClip);
  if (reference.twitchChannel) params.set("channel", reference.twitchChannel);
  for (const parent of parents) params.append("parent", parent);
  params.set("autoplay", "false");
  if (reference.twitchClip) return `https://clips.twitch.tv/embed?${params}`;
  return `https://player.twitch.tv/?${params}`;
}

export function normalizeTwitchParents(input: string | string[] | undefined): string[] {
  const values = Array.isArray(input) ? input : (input?.split(",") ?? []);
  return Array.from(new Set(values.map((value) => value.trim().toLowerCase()).filter(safeParent)));
}

function vimeoReference(url: URL, segments: string[]): VideoProviderReference | null {
  const host = url.hostname.toLowerCase();
  const id =
    host === "player.vimeo.com" && segments[0] === "video"
      ? safeNumericId(segments[1])
      : ["vimeo.com", "www.vimeo.com"].includes(host)
        ? vimeoVideoId(segments)
        : undefined;
  if (!id) return null;
  const canonicalUrl = `https://vimeo.com/${id}`;
  return {
    provider: "vimeo",
    kind: "video",
    canonicalUrl,
    title: `Vimeo video ${id}`,
    apiUrl: `https://vimeo.com/api/oembed.json?url=${encodeURIComponent(canonicalUrl)}`,
    vimeoId: id,
  };
}

function twitchReference(url: URL, segments: string[]): VideoProviderReference | null {
  const host = url.hostname.toLowerCase();
  if (host === "clips.twitch.tv") {
    const slug = safeSlug(segments[0]);
    return slug
      ? {
          provider: "twitch",
          kind: "clip",
          canonicalUrl: `https://clips.twitch.tv/${slug}`,
          title: titleize(slug),
          twitchClip: slug,
        }
      : null;
  }
  if (!["twitch.tv", "www.twitch.tv"].includes(host) || segments.length === 0) return null;
  if (segments[0] === "videos") {
    const id = safeNumericId(segments[1]);
    return id
      ? {
          provider: "twitch",
          kind: "video",
          canonicalUrl: `https://www.twitch.tv/videos/${id}`,
          title: `Twitch video ${id}`,
          twitchVideoId: id,
        }
      : null;
  }
  const channel = safeChannel(segments[0]);
  if (!channel || TWITCH_RESERVED_CHANNELS.has(channel.toLowerCase())) return null;
  if (segments[1] === "clip") {
    const slug = safeSlug(segments[2]);
    return slug
      ? {
          provider: "twitch",
          kind: "clip",
          canonicalUrl: `https://www.twitch.tv/${channel}/clip/${slug}`,
          title: titleize(slug),
          author: channel,
          twitchClip: slug,
        }
      : null;
  }
  return segments.length === 1
    ? {
        provider: "twitch",
        kind: "channel",
        canonicalUrl: `https://www.twitch.tv/${channel}`,
        title: `${channel} on Twitch`,
        author: channel,
        twitchChannel: channel,
      }
    : null;
}

function vimeoVideoId(segments: string[]): string | undefined {
  if (segments.length === 1) return safeNumericId(segments[0]);
  const videoIndex = segments.indexOf("video");
  if (videoIndex >= 0) return safeNumericId(segments[videoIndex + 1]);
  return segments.length <= 3 ? safeNumericId(segments.at(-1)) : undefined;
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? decodeProviderArticleAttr(value) : undefined;
}

function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function safeParent(value: string): boolean {
  return (
    value === "localhost" ||
    value === "127.0.0.1" ||
    (value.length <= 253 && /^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/.test(value) && value.includes("."))
  );
}

function safeNumericId(value: string | undefined): string | undefined {
  return value && /^[0-9]{1,32}$/.test(value) ? value : undefined;
}

function safeChannel(value: string | undefined): string | undefined {
  return value && /^[A-Za-z0-9_]{1,25}$/.test(value) ? value : undefined;
}

function safeSlug(value: string | undefined): string | undefined {
  return value && /^[A-Za-z0-9][A-Za-z0-9_-]{0,127}$/.test(value) ? value : undefined;
}

function titleize(value: string): string {
  return value.replaceAll("-", " ").replaceAll("_", " ");
}
