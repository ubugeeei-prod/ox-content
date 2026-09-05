import { renderDocumentAssets as renderDocumentAssetsImpl } from "./document-assets-runtime";
import { renderDocumentAssetTag as renderDocumentAssetTagImpl } from "./document-assets-tags";

export type DocumentCrossOrigin = true | "anonymous" | "use-credentials";
export type DocumentAssetAttributes = Record<string, string | number | boolean | undefined | null>;

export interface DocumentAssetCommon {
  /**
   * Stable identity used for dedupe. When omitted, href/src/content identity is
   * used.
   */
  key?: string;
  /** Extra attributes written after the typed fields. */
  attrs?: DocumentAssetAttributes;
}

export interface DocumentLinkDescriptor extends DocumentAssetCommon {
  kind: "link";
  rel: string;
  href: string;
  as?: string;
  type?: string;
  media?: string;
  sizes?: string;
  title?: string;
  integrity?: string;
  referrerpolicy?: string;
  fetchpriority?: "high" | "low" | "auto" | (string & {});
  crossorigin?: DocumentCrossOrigin;
  nonce?: string;
}

export interface DocumentStyleDescriptor extends DocumentAssetCommon {
  kind: "style";
  href?: string;
  content?: string;
  media?: string;
  title?: string;
  integrity?: string;
  crossorigin?: DocumentCrossOrigin;
  nonce?: string;
}

export interface DocumentScriptDescriptor extends DocumentAssetCommon {
  kind: "script";
  src?: string;
  content?: string;
  type?: string;
  async?: boolean;
  defer?: boolean;
  integrity?: string;
  crossorigin?: DocumentCrossOrigin;
  nonce?: string;
}

export type DocumentAssetDescriptor =
  | DocumentLinkDescriptor
  | DocumentStyleDescriptor
  | DocumentScriptDescriptor;

export type DocumentStylesheetInput =
  | string
  | Omit<DocumentStyleDescriptor, "kind">
  | DocumentStyleDescriptor;
export type DocumentScriptInput =
  | string
  | Omit<DocumentScriptDescriptor, "kind">
  | DocumentScriptDescriptor;
export type DocumentLinkInput =
  | string
  | Omit<DocumentLinkDescriptor, "kind">
  | DocumentLinkDescriptor;

export interface DocumentAssetManifestChunk {
  file?: string;
  src?: string;
  css?: string[];
  imports?: string[];
  isEntry?: boolean;
}

export type DocumentAssetManifest = Record<string, DocumentAssetManifestChunk>;

export interface DocumentSelfHostedAssets {
  stylesheets?: readonly string[];
  preloads?: readonly {
    href: string;
    as: string;
    type?: string;
    crossorigin?: DocumentCrossOrigin;
  }[];
}

export interface DocumentAssetNoncePolicy {
  style?: string;
  script?: string;
}

export interface RenderDocumentAssetsInput {
  /** Base path for Vite manifest entries and host-declared absolute assets. */
  base?: string;
  /** Existing metadata head markup, for example `renderHead(input).html`. */
  head?: string | { html?: string } | null;
  /** Vite manifest from `.vite/manifest.json` for build mode. */
  manifest?: DocumentAssetManifest;
  /** Structured output from `resolveSelfHostedAssetManifest()`. */
  selfHostedAssets?: DocumentSelfHostedAssets | null;
  /** Generic links such as canonical, manifest, preconnect, or favicons. */
  links?: readonly DocumentLinkInput[];
  /** Styles shared by every page in the custom host. */
  sharedStyles?: readonly DocumentStylesheetInput[];
  /** Styles selected for one route/page. */
  pageStyles?: readonly DocumentStylesheetInput[];
  /** SSR-visible island/component styles selected for one route/page. */
  islandStyles?: readonly DocumentStylesheetInput[];
  /** Inline critical CSS selected by the host for one route/page. */
  inlineStyles?: readonly DocumentStylesheetInput[];
  /** Client entries. Build mode resolves these through `manifest` when present. */
  clientEntries?: readonly DocumentScriptInput[];
  /** Extra scripts appended after client entries. */
  scripts?: readonly DocumentScriptInput[];
  /** CSP nonce policy applied when an individual descriptor does not set one. */
  nonce?: string | DocumentAssetNoncePolicy;
  /** Crossorigin policy applied to manifest client entries. */
  crossorigin?: DocumentCrossOrigin;
}

export interface RenderDocumentAssetsResult {
  links: DocumentLinkDescriptor[];
  styles: DocumentStyleDescriptor[];
  scripts: DocumentScriptDescriptor[];
  tags: DocumentAssetDescriptor[];
  headHtml: string;
}

export function renderDocumentAssetTag(tag: DocumentAssetDescriptor): string {
  return renderDocumentAssetTagImpl(tag);
}

export function renderDocumentAssets(input: RenderDocumentAssetsInput): RenderDocumentAssetsResult {
  return renderDocumentAssetsImpl(input);
}
