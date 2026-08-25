import { extractCodeBlocks } from "./code-blocks";
import {
  generateTypedHoverAttachments,
  type TypedHoverAttachment,
  type TypedHoverRange,
} from "./typed-hover-generate";
import type { OxContentOptions, ResolvedOptions, ResolvedTypedHoverOptions } from "./types";

export type { TypedHoverAttachment, TypedHoverRange };

export interface TypedHoverPayload {
  hovers: TypedHoverRange[];
}

const DEFAULT_LANGUAGES = ["ts", "tsx"] as const;

export function resolveTypedHoverOptions(
  options: OxContentOptions["typedHover"],
): ResolvedTypedHoverOptions {
  if (!options) {
    return { enabled: false, languages: [...DEFAULT_LANGUAGES] };
  }
  if (options === true) {
    return { enabled: true, languages: [...DEFAULT_LANGUAGES] };
  }
  return {
    enabled: options.enabled ?? true,
    languages: options.languages ?? [...DEFAULT_LANGUAGES],
    tsgoCommand: options.tsgoCommand,
  };
}

export function hasTypedHoverMeta(meta: string): boolean {
  return meta.split(/\s+/).some((token) => token === "twoslash");
}

export function serializeTypedHoverPayload(payload: TypedHoverPayload): string {
  return JSON.stringify(payload).replace(/</g, "\\u003c").replace(/>/g, "\\u003e");
}

export async function applyTypedHover(
  source: string,
  html: string,
  options: ResolvedOptions["typedHover"],
): Promise<string> {
  if (!options?.enabled || !source.includes("```")) {
    return html;
  }

  const languages = new Set(options.languages.map((language) => language.toLowerCase()));
  const fences = (await extractCodeBlocks(source)).filter((block) => {
    return languages.has(block.language.toLowerCase()) && hasTypedHoverMeta(block.meta);
  });
  if (fences.length === 0) {
    return html;
  }

  try {
    const attachments = await generateTypedHoverAttachments(fences, options.tsgoCommand);
    return attachTypedHoverPayloads(html, attachments);
  } catch (error) {
    console.warn("[ox-content] typedHover failed; leaving the fence unannotated.", error);
    return html;
  }
}

export function attachTypedHoverPayloads(
  html: string,
  attachments: TypedHoverAttachment[],
): string {
  const unused = attachments.filter((item) => item.hovers.length > 0);
  if (unused.length === 0) {
    return html;
  }

  let attached = 0;
  const next = html.replace(
    /<pre(\b[^>]*)><code(\b[^>]*)>([\s\S]*?)<\/code><\/pre>/g,
    (full, preAttrs: string, codeAttrs: string, inner: string) => {
      if (unused.length === 0) {
        return full;
      }
      if (!isTypeScriptFence(codeAttrs)) {
        return full;
      }
      const text = decodeHtmlEntities(inner.replace(/<[^>]+>/g, ""));
      const index = unused.findIndex(
        (item) => normalizeFenceText(item.code) === normalizeFenceText(text),
      );
      if (index === -1) {
        return full;
      }
      const item = unused.splice(index, 1)[0];
      if (!item) {
        return full;
      }
      attached += 1;
      const wrapped = wrapHoverRanges(inner, item.hovers);
      return `${withTypedHoverClass(`<pre${preAttrs}`)}><code${codeAttrs}>${wrapped}</code></pre>\n<script type="application/json" class="ox-typed-hover-data">${serializeTypedHoverPayload({ hovers: item.hovers })}</script>`;
    },
  );

  if (attached === 0) {
    return html;
  }
  return `${next}${TYPED_HOVER_STYLE}${TYPED_HOVER_CLIENT}`;
}

function isTypeScriptFence(codeAttrs: string): boolean {
  const match = codeAttrs.match(/class="([^"]*)"/);
  if (!match?.[1]) {
    return false;
  }
  return match[1].split(/\s+/).some((token) => {
    const language = token.replace(/^language-/, "").toLowerCase();
    return (
      language === "ts" ||
      language === "tsx" ||
      language === "typescript" ||
      language === "typescriptreact"
    );
  });
}

function withTypedHoverClass(openPre: string): string {
  if (/\bclass="/.test(openPre)) {
    return openPre.replace(/\bclass="([^"]*)"/, (_, classes: string) => {
      return `class="${classes} ox-typed-hover"`;
    });
  }
  return `${openPre} class="ox-typed-hover"`;
}

function wrapHoverRanges(inner: string, hovers: TypedHoverRange[]): string {
  const ranges = [...hovers].sort((a, b) => a.start - b.start || b.end - a.end);
  let output = "";
  let htmlIndex = 0;
  let sourceOffset = 0;
  let rangeIndex = 0;
  let openUntil = -1;
  let openHoverIndex = -1;

  const startRange = (): void => {
    while (rangeIndex < ranges.length && ranges[rangeIndex]!.start < sourceOffset) {
      rangeIndex += 1;
    }
    const range = ranges[rangeIndex];
    if (!range || range.start !== sourceOffset || openUntil !== -1) {
      return;
    }
    output += `<span class="ox-typed-hover-token" tabindex="0" data-ox-typed-hover="${rangeIndex}">`;
    openUntil = range.end;
    openHoverIndex = rangeIndex;
    rangeIndex += 1;
  };

  const endRange = (): void => {
    if (openUntil === sourceOffset && openHoverIndex !== -1) {
      output += "</span>";
      openUntil = -1;
      openHoverIndex = -1;
    }
  };

  while (htmlIndex < inner.length) {
    const char = inner[htmlIndex]!;
    if (char === "<") {
      const close = inner.indexOf(">", htmlIndex);
      const tag = close === -1 ? inner.slice(htmlIndex) : inner.slice(htmlIndex, close + 1);
      output += tag;
      htmlIndex += tag.length;
      continue;
    }

    startRange();
    if (char === "&") {
      const semi = inner.indexOf(";", htmlIndex);
      const entity = semi === -1 ? inner.slice(htmlIndex) : inner.slice(htmlIndex, semi + 1);
      output += entity;
      htmlIndex += entity.length;
      sourceOffset += 1;
      endRange();
      continue;
    }

    output += char;
    htmlIndex += 1;
    sourceOffset += 1;
    endRange();
  }

  if (openHoverIndex !== -1) {
    output += "</span>";
  }
  return output;
}

function normalizeFenceText(value: string): string {
  return decodeHtmlEntities(value).replace(/\r\n/g, "\n").trim();
}

function decodeHtmlEntities(value: string): string {
  return value
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&#x([0-9a-fA-F]+);/g, (_, hex: string) => {
      const code = Number.parseInt(hex, 16);
      return Number.isFinite(code) ? String.fromCodePoint(code) : "";
    })
    .replace(/&#(\d+);/g, (_, dec: string) => {
      const code = Number.parseInt(dec, 10);
      return Number.isFinite(code) ? String.fromCodePoint(code) : "";
    })
    .replace(/&amp;/g, "&");
}

const TYPED_HOVER_STYLE = `<style data-ox-typed-hover-style>.ox-typed-hover-token{cursor:help;text-decoration:underline dotted}.ox-typed-hover-overlay{position:fixed;z-index:50;max-width:36rem;padding:.35rem .55rem;border:1px solid #444;border-radius:4px;background:#1e1e1e;color:#d4d4d4;font:12px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;pointer-events:none}</style>`;

const TYPED_HOVER_CLIENT = `<script data-ox-typed-hover-runtime>(function(){if(window.__oxTypedHover)return;window.__oxTypedHover=1;var tip=document.createElement("div");tip.className="ox-typed-hover-overlay";tip.setAttribute("role","tooltip");tip.hidden=true;document.body.appendChild(tip);function closest(value,selector){return value instanceof Element?value.closest(selector):null}function nextData(host){var node=host.nextElementSibling;while(node&&node.tagName==="SCRIPT"&&!node.classList.contains("ox-typed-hover-data"))node=node.nextElementSibling;return node}function payload(token){var pre=token.closest(".ox-typed-hover");if(!pre)return null;var host=pre.closest(".ox-code")||pre;var data=nextData(host)||nextData(pre);if(!data||!data.classList.contains("ox-typed-hover-data"))return null;try{return JSON.parse(data.textContent||"")}catch(e){return null}}function show(token){var data=payload(token);var item=data&&data.hovers[Number(token.getAttribute("data-ox-typed-hover"))];if(!item)return;tip.textContent=item.type;tip.hidden=false;var box=token.getBoundingClientRect();tip.style.left=Math.max(8,box.left)+"px";tip.style.top=Math.max(8,box.top-tip.offsetHeight-8)+"px"}function hide(){tip.hidden=true}document.addEventListener("mouseover",function(e){var t=closest(e.target,".ox-typed-hover-token");if(t)show(t)});document.addEventListener("mouseout",function(e){var t=closest(e.target,".ox-typed-hover-token");if(t&&!t.contains(e.relatedTarget))hide()});document.addEventListener("focusin",function(e){var t=closest(e.target,".ox-typed-hover-token");if(t)show(t)});document.addEventListener("focusout",function(e){if(!closest(e.relatedTarget,".ox-typed-hover-token"))hide()});document.addEventListener("keydown",function(e){if(e.key==="Escape")hide()})})();</script>`;
