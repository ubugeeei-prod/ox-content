/// <reference lib="dom" />

import {
  DEFAULT_ATOM_CLASS,
  DEFAULT_CHAR_CLASS,
  DEFAULT_VISIBLE_CLASS,
} from "./incremental-dom-styles";
import type { IncrementalMarkdownDomAnimationOptions } from "./incremental-dom-types";

export interface ResolvedAnimationOptions {
  mode: "none" | "character";
  durationMs: number;
  staggerMs: number;
  committed: boolean;
  pending: boolean;
  charClassName: string;
  visibleClassName: string;
  atomClassName: string;
}

export function resolveAnimationOptions(
  animation: boolean | IncrementalMarkdownDomAnimationOptions | undefined,
): ResolvedAnimationOptions {
  if (!animation) {
    return {
      mode: "none",
      durationMs: 0,
      staggerMs: 0,
      committed: false,
      pending: false,
      charClassName: DEFAULT_CHAR_CLASS,
      visibleClassName: DEFAULT_VISIBLE_CLASS,
      atomClassName: DEFAULT_ATOM_CLASS,
    };
  }

  const options = animation === true ? {} : animation;
  return {
    mode: options.mode ?? "character",
    durationMs: options.durationMs ?? 170,
    staggerMs: options.staggerMs ?? 2,
    committed: options.committed ?? true,
    pending: options.pending ?? true,
    charClassName: options.charClassName ?? DEFAULT_CHAR_CLASS,
    visibleClassName: options.visibleClassName ?? DEFAULT_VISIBLE_CLASS,
    atomClassName: options.atomClassName ?? DEFAULT_ATOM_CLASS,
  };
}

export function prepareCharacterFade(
  document: Document,
  root: ParentNode,
  animation: ResolvedAnimationOptions,
  visibleTextPrefix: number,
): void {
  if (animation.mode === "none") {
    return;
  }

  const textNodes = collectTextNodes(document, root);
  let index = 0;
  let textOffset = 0;

  for (const textNode of textNodes) {
    const value = textNode.nodeValue ?? "";
    const parent = textNode.parentElement;
    const preserveWhitespace = Boolean(parent?.closest("pre, code"));

    if (!value || (!preserveWhitespace && !value.trim())) {
      continue;
    }

    const fragment = document.createDocumentFragment();
    for (const char of [...value]) {
      const span = document.createElement("span");
      if (textOffset < visibleTextPrefix) {
        span.className = `${animation.charClassName} ${animation.visibleClassName}`;
      } else {
        span.className = animation.charClassName;
        span.style.setProperty("--ox-incremental-delay", `${index * animation.staggerMs}ms`);
        span.style.setProperty("--ox-incremental-duration", `${animation.durationMs}ms`);
        index += 1;
      }
      span.textContent = char;
      fragment.append(span);
      textOffset += 1;
    }
    textNode.replaceWith(fragment);
  }

  for (const atom of root.querySelectorAll("input, img, hr, br, iframe, svg")) {
    if (textOffset < visibleTextPrefix) {
      continue;
    }
    const element = atom as HTMLElement;
    atom.classList.add(animation.atomClassName);
    element.style.setProperty("--ox-incremental-delay", `${index * animation.staggerMs}ms`);
    element.style.setProperty("--ox-incremental-duration", `${animation.durationMs}ms`);
    index += 1;
  }
}

function collectTextNodes(document: Document, root: ParentNode): Text[] {
  const textNodes: Text[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);

  while (walker.nextNode()) {
    textNodes.push(walker.currentNode as Text);
  }

  return textNodes;
}
