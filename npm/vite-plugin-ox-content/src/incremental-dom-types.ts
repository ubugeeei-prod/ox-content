/// <reference lib="dom" />

export interface IncrementalMarkdownRenderResultLike {
  deltaHtml: string;
  committedHtml: string;
  pendingHtml: string;
  html: string;
  committedBytes: number;
  pendingBytes: number;
  totalBytes: number;
  didCommit: boolean;
  isFinal: boolean;
  errors: string[];
}

export interface IncrementalMarkdownDomAnimationOptions {
  /**
   * Animate newly-seen text one character at a time.
   * @default "character"
   */
  mode?: "none" | "character";
  /**
   * Duration for each character fade.
   * @default 170
   */
  durationMs?: number;
  /**
   * Delay added per new character.
   * @default 2
   */
  staggerMs?: number;
  /**
   * Animate newly committed HTML.
   * @default true
   */
  committed?: boolean;
  /**
   * Animate newly appended pending text. Existing pending prefixes are not re-animated.
   * @default true
   */
  pending?: boolean;
  /**
   * CSS class for animated characters.
   * @default "ox-incremental-char"
   */
  charClassName?: string;
  /**
   * CSS class for characters that have already been visible in the pending region.
   * @default "ox-incremental-char--visible"
   */
  visibleClassName?: string;
  /**
   * CSS class for non-text atoms such as checkboxes, images, and rules.
   * @default "ox-incremental-atom"
   */
  atomClassName?: string;
}

export interface IncrementalMarkdownDomOptions {
  /** Element that owns the committed and pending render regions. */
  preview: Element;
  /** Optional element that displays the raw Markdown stream. */
  source?: HTMLElement;
  /**
   * Character fade is opt-in. Pass `true` for defaults or an option object.
   * @default false
   */
  animation?: boolean | IncrementalMarkdownDomAnimationOptions;
  /**
   * CSS class for the committed render root.
   * @default "ox-incremental-committed"
   */
  committedClassName?: string;
  /**
   * CSS class for the replaceable pending render root.
   * @default "ox-incremental-pending"
   */
  pendingClassName?: string;
}

export interface IncrementalMarkdownDomApplyOptions {
  /** Raw Markdown chunk to append to `source`, when one is configured. */
  chunk?: string;
}

export interface IncrementalMarkdownDomRenderer {
  readonly committedRoot: HTMLElement;
  readonly pendingRoot: HTMLElement;
  apply(
    result: IncrementalMarkdownRenderResultLike,
    options?: IncrementalMarkdownDomApplyOptions,
  ): void;
  reset(): void;
}
