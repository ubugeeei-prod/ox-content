import type {
  DocumentAssetAttributes,
  DocumentAssetDescriptor,
  RenderDocumentAssetsInput,
} from "./document-assets";

export function renderDocumentAssetTag(tag: DocumentAssetDescriptor): string {
  if (tag.kind === "link") {
    return renderEmptyTag("link", [
      ["rel", tag.rel],
      ["href", tag.href],
      ["as", tag.as],
      ["type", tag.type],
      ["media", tag.media],
      ["sizes", tag.sizes],
      ["title", tag.title],
      ["integrity", tag.integrity],
      ["referrerpolicy", tag.referrerpolicy],
      ["fetchpriority", tag.fetchpriority],
      ["crossorigin", tag.crossorigin],
      ["nonce", tag.nonce],
      ...attributeEntries(tag.attrs),
    ]);
  }
  if (tag.kind === "style" && tag.href) {
    return renderEmptyTag("link", [
      ["rel", "stylesheet"],
      ["href", tag.href],
      ["media", tag.media],
      ["title", tag.title],
      ["integrity", tag.integrity],
      ["crossorigin", tag.crossorigin],
      ["nonce", tag.nonce],
      ...attributeEntries(tag.attrs),
    ]);
  }
  if (tag.kind === "style") {
    return renderPairedTag(
      "style",
      [
        ["media", tag.media],
        ["title", tag.title],
        ["nonce", tag.nonce],
        ...attributeEntries(tag.attrs),
      ],
      escapeStyleText(tag.content ?? ""),
    );
  }
  const type = tag.type ?? (tag.src ? "module" : undefined);
  return renderPairedTag(
    "script",
    [
      ["type", type],
      ["src", tag.src],
      ["async", tag.async],
      ["defer", tag.defer],
      ["integrity", tag.integrity],
      ["crossorigin", tag.crossorigin],
      ["nonce", tag.nonce],
      ...attributeEntries(tag.attrs),
    ],
    escapeScriptText(tag.content ?? ""),
  );
}

export function headHtml(head: RenderDocumentAssetsInput["head"]): string {
  if (!head) {
    return "";
  }
  if (typeof head === "string") {
    return head.trim();
  }
  return head.html?.trim() ?? "";
}

type AttributeEntry = readonly [string, string | number | boolean | undefined | null];

function renderEmptyTag(name: string, attrs: AttributeEntry[]): string {
  return `<${name}${renderAttrs(attrs)}>`;
}

function renderPairedTag(name: string, attrs: AttributeEntry[], content: string): string {
  return `<${name}${renderAttrs(attrs)}>${content}</${name}>`;
}

function renderAttrs(attrs: AttributeEntry[]): string {
  const rendered = attrs.flatMap(([name, value]) => renderAttr(name, value));
  return rendered.length > 0 ? ` ${rendered.join(" ")}` : "";
}

function renderAttr(name: string, value: string | number | boolean | undefined | null): string[] {
  if (value === false || value === undefined || value === null) {
    return [];
  }
  if (value === true) {
    return [name];
  }
  return [`${name}="${escapeAttr(String(value))}"`];
}

function attributeEntries(attrs: DocumentAssetAttributes | undefined): AttributeEntry[] {
  return Object.entries(attrs ?? {});
}

function escapeAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function escapeStyleText(value: string): string {
  return value.replace(/<\/style/giu, (match) => `<\\/${match.slice(2)}`);
}

function escapeScriptText(value: string): string {
  return value.replace(/<\/script/giu, (match) => `<\\/${match.slice(2)}`);
}
