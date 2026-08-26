import type { PackageRegistryReference } from "./provider-packages";

export interface PackageMeta {
  title?: string;
  description?: string;
  version?: string;
  license?: string;
  repository?: string;
  downloads?: string;
  stars?: string;
  dateTime?: string;
  dateLabel?: string;
}

export function packageMetaFromJson(
  value: unknown,
  reference: PackageRegistryReference,
): PackageMeta | null {
  switch (reference.provider) {
    case "npm":
      return npmMeta(value, reference);
    case "crates.io":
      return cratesMeta(value, reference);
    case "pypi":
      return pypiMeta(value, reference);
    case "docker-hub":
      return dockerMeta(value, reference);
  }
}

function npmMeta(value: unknown, reference: PackageRegistryReference): PackageMeta | null {
  const item = record(value);
  const title = trimmed(item?.name) ?? reference.name;
  const versions = record(item?.versions);
  const latest = trimmed(record(item?.["dist-tags"])?.latest);
  const version = reference.version ?? latest;
  const versionData = version ? record(versions?.[version]) : undefined;
  const time = record(item?.time);
  const dateTime = version ? trimmed(time?.[version]) : trimmed(time?.modified);
  return compactMeta({
    title,
    version: trimmed(versionData?.version) ?? version,
    description: trimmed(versionData?.description) ?? trimmed(item?.description),
    license: licenseField(versionData?.license) ?? licenseField(item?.license),
    repository: repositoryField(versionData?.repository ?? item?.repository),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function cratesMeta(value: unknown, reference: PackageRegistryReference): PackageMeta | null {
  const item = record(record(value)?.crate);
  const title = trimmed(item?.name) ?? reference.name;
  const dateTime = trimmed(item?.updated_at);
  return compactMeta({
    title,
    version: reference.version ?? trimmed(item?.max_version) ?? trimmed(item?.newest_version),
    description: trimmed(item?.description),
    license: licenseField(item?.license),
    repository: safeDisplayUrl(trimmed(item?.repository)),
    downloads: count(item?.downloads),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function pypiMeta(value: unknown, reference: PackageRegistryReference): PackageMeta | null {
  const root = record(value);
  const info = record(root?.info);
  const version = reference.version ?? trimmed(info?.version);
  const releaseItems = reference.version ? root?.urls : record(root?.releases)?.[version];
  const release = version ? record(firstArrayItem(releaseItems)) : undefined;
  const dateTime = trimmed(release?.upload_time_iso_8601);
  return compactMeta({
    title: trimmed(info?.name) ?? reference.name,
    version,
    description: trimmed(info?.summary),
    license: licenseField(info?.license),
    repository: projectUrl(record(info?.project_urls)),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function dockerMeta(value: unknown, reference: PackageRegistryReference): PackageMeta | null {
  const item = record(value);
  const title = [trimmed(item?.namespace), trimmed(item?.name)].filter(Boolean).join("/");
  const dateTime = trimmed(item?.last_updated);
  return compactMeta({
    title: title || reference.name,
    version: reference.version,
    description: trimmed(item?.description) ?? trimmed(item?.short_description),
    downloads: count(item?.pull_count),
    stars: count(item?.star_count),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function firstArrayItem(value: unknown): unknown {
  return Array.isArray(value) ? value[0] : undefined;
}

function licenseField(value: unknown): string | undefined {
  const text = typeof value === "string" ? value : trimmed(record(value)?.type);
  return text && text !== "UNKNOWN" ? text : undefined;
}

function repositoryField(value: unknown): string | undefined {
  const text = typeof value === "string" ? value : trimmed(record(value)?.url);
  return safeDisplayUrl(text?.replace(/^git\+/, "").replace(/\.git$/, ""));
}

function projectUrl(urls: Record<string, unknown> | undefined): string | undefined {
  for (const key of ["Source", "Repository", "Source Code", "Homepage", "Home"]) {
    const value = safeDisplayUrl(trimmed(urls?.[key]));
    if (value) return value;
  }
  return undefined;
}

function safeDisplayUrl(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

function count(value: unknown): string | undefined {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? String(Math.floor(value))
    : undefined;
}

function dateLabel(value: string | undefined): string | undefined {
  return value && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : undefined;
}

function compactMeta(meta: PackageMeta): PackageMeta | null {
  const compact = Object.fromEntries(Object.entries(meta).filter(([, value]) => value));
  return Object.keys(compact).length ? (compact as PackageMeta) : null;
}

function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
