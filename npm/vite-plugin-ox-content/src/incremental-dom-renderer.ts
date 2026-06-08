/// <reference lib="dom" />

import { prepareCharacterFade, resolveAnimationOptions } from "./incremental-dom-animation";
import { DEFAULT_COMMITTED_CLASS, DEFAULT_PENDING_CLASS } from "./incremental-dom-styles";
import type {
  IncrementalMarkdownDomApplyOptions,
  IncrementalMarkdownDomOptions,
  IncrementalMarkdownDomRenderer,
  IncrementalMarkdownRenderResultLike,
} from "./incremental-dom-types";

export function createIncrementalMarkdownDomRenderer(
  options: IncrementalMarkdownDomOptions,
): IncrementalMarkdownDomRenderer {
  const document = options.preview.ownerDocument;
  const animation = resolveAnimationOptions(options.animation);
  const committedRoot = document.createElement("div");
  const pendingRoot = document.createElement("div");
  const committedClassName = options.committedClassName ?? DEFAULT_COMMITTED_CLASS;
  const pendingClassName = options.pendingClassName ?? DEFAULT_PENDING_CLASS;
  let markdown = "";

  committedRoot.className = committedClassName;
  pendingRoot.className = pendingClassName;
  options.preview.replaceChildren(committedRoot, pendingRoot);

  function apply(
    result: IncrementalMarkdownRenderResultLike,
    applyOptions: IncrementalMarkdownDomApplyOptions = {},
  ): void {
    const previousPendingText = pendingRoot.textContent ?? "";

    if (applyOptions.chunk !== undefined && options.source) {
      markdown += applyOptions.chunk;
      options.source.textContent = markdown;
    }

    if (result.deltaHtml) {
      appendCommittedHtml(result.deltaHtml, previousPendingText);
    }

    setPendingHtml(result.pendingHtml);
  }

  function reset(): void {
    markdown = "";
    if (options.source) {
      options.source.textContent = "";
    }
    committedRoot.replaceChildren();
    pendingRoot.replaceChildren();
  }

  function appendCommittedHtml(html: string, previousPendingText: string): void {
    const fragment = parseHtml(html);
    const committedText = fragment.textContent ?? "";
    const visiblePrefix = commonPrefixLength(previousPendingText, committedText);
    prepareCharacterFade(
      document,
      fragment,
      animation,
      animation.committed ? visiblePrefix : Infinity,
    );
    committedRoot.append(fragment);
  }

  function setPendingHtml(html: string): void {
    const previousText = pendingRoot.textContent ?? "";
    const fragment = parseHtml(html);
    const nextText = fragment.textContent ?? "";
    const visiblePrefix = animation.pending ? commonPrefixLength(previousText, nextText) : Infinity;
    prepareCharacterFade(document, fragment, animation, visiblePrefix);
    pendingRoot.replaceChildren(fragment);
  }

  function parseHtml(html: string): DocumentFragment {
    const template = document.createElement("template");
    template.innerHTML = html;
    return template.content;
  }

  return { committedRoot, pendingRoot, apply, reset };
}

function commonPrefixLength(left: string, right: string): number {
  const max = Math.min(left.length, right.length);
  let index = 0;

  while (index < max && left[index] === right[index]) {
    index += 1;
  }

  return index;
}
