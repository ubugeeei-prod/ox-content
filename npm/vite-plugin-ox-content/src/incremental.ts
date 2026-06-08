import { importNapiModuleSync } from "./napi";

type NapiModule = typeof import("@ox-content/napi");
type NativeIncrementalMarkdownParser = InstanceType<NapiModule["IncrementalMarkdownParser"]>;
type NativeIncrementalMarkdownRenderer = InstanceType<NapiModule["IncrementalMarkdownRenderer"]>;
type NativeParseResult = import("@ox-content/napi").IncrementalMarkdownParseResult;

export interface IncrementalMarkdownParserOptions {
  /**
   * Enable GitHub Flavored Markdown extensions.
   * @default true
   */
  gfm?: boolean;
  /** Enable footnotes. */
  footnotes?: boolean;
  /** Enable task list items. */
  taskLists?: boolean;
  /** Enable GFM tables. */
  tables?: boolean;
  /** Enable strikethrough. */
  strikethrough?: boolean;
  /** Enable Markdown autolinks. */
  autolinks?: boolean;
}

export interface IncrementalMarkdownParseAppendOptions {
  /** Commit the current chunk as the final stream input. */
  final?: boolean;
  /** Include a provisional AST for the current replaceable tail. */
  includePendingAst?: boolean;
  /** Temporarily close unmatched inline delimiters in the provisional AST. */
  completeInline?: boolean;
}

export interface IncrementalMarkdownRendererOptions extends IncrementalMarkdownParserOptions {
  /**
   * Render the unstable tail as replaceable provisional HTML.
   * @default true
   */
  renderPending?: boolean;
  /**
   * Temporarily close unmatched inline delimiters in provisional HTML.
   * @default true
   */
  completeInline?: boolean;
}

export interface IncrementalMarkdownRenderAppendOptions {
  /** Commit the current chunk as the final stream input. */
  final?: boolean;
  /** Render the unstable tail as replaceable provisional HTML. */
  renderPending?: boolean;
  /** Temporarily close unmatched inline delimiters in provisional HTML. */
  completeInline?: boolean;
}

export type IncrementalMarkdownRenderResult =
  import("@ox-content/napi").IncrementalMarkdownRenderResult;

export interface IncrementalMarkdownParseResult<TAst = unknown>
  extends Omit<NativeParseResult, "ast" | "pendingAst"> {
  /** Parsed mdast for the newly committed Markdown prefix, or null when nothing committed. */
  ast: TAst | null;
  /** Raw mdast JSON for the newly committed Markdown prefix. */
  astJson: string;
  /** Provisional parsed mdast for the current replaceable tail, or null when not requested. */
  pendingAst: TAst | null;
  /** Raw provisional mdast JSON for the current replaceable tail. */
  pendingAstJson: string;
}

export type MarkdownChunkSource = Iterable<string> | AsyncIterable<string>;

function toNativeParserOptions(options: IncrementalMarkdownParserOptions = {}) {
  return {
    gfm: options.gfm ?? true,
    footnotes: options.footnotes,
    taskLists: options.taskLists,
    tables: options.tables,
    strikethrough: options.strikethrough,
    autolinks: options.autolinks,
  };
}

function parseAstJson<TAst>(json: string): TAst | null {
  return json ? (JSON.parse(json) as TAst) : null;
}

function normalizeParseResult<TAst>(
  result: NativeParseResult,
): IncrementalMarkdownParseResult<TAst> {
  const { ast, pendingAst, ...rest } = result;
  return {
    ...rest,
    ast: parseAstJson<TAst>(ast),
    astJson: ast,
    pendingAst: parseAstJson<TAst>(pendingAst),
    pendingAstJson: pendingAst,
  };
}

export class IncrementalMarkdownParser<TAst = unknown> {
  readonly #native: NativeIncrementalMarkdownParser;
  readonly #includePendingAst: boolean;
  readonly #completeInline: boolean;

  constructor(options: IncrementalMarkdownParserOptions & IncrementalMarkdownParseAppendOptions = {}) {
    const napi = importNapiModuleSync();
    this.#native = new napi.IncrementalMarkdownParser(toNativeParserOptions(options));
    this.#includePendingAst = options.includePendingAst ?? false;
    this.#completeInline = options.completeInline ?? true;
  }

  append(
    chunk: string,
    options: IncrementalMarkdownParseAppendOptions = {},
  ): IncrementalMarkdownParseResult<TAst> {
    return normalizeParseResult<TAst>(
      this.#native.append(chunk, {
        isFinal: options.final ?? false,
        includePendingAst: options.includePendingAst ?? this.#includePendingAst,
        completeInline: options.completeInline ?? this.#completeInline,
      }),
    );
  }

  finish(options: IncrementalMarkdownParseAppendOptions = {}): IncrementalMarkdownParseResult<TAst> {
    return normalizeParseResult<TAst>(
      this.#native.finish({
        includePendingAst: options.includePendingAst ?? this.#includePendingAst,
        completeInline: options.completeInline ?? this.#completeInline,
      }),
    );
  }

  reset(): void {
    this.#native.reset();
  }

  get pendingMarkdown(): string {
    return this.#native.pendingMarkdown;
  }

  get committedBytes(): number {
    return this.#native.committedBytes;
  }

  get totalBytes(): number {
    return this.#native.totalBytes;
  }
}

export class IncrementalMarkdownRenderer {
  readonly #native: NativeIncrementalMarkdownRenderer;
  readonly #renderPending: boolean;
  readonly #completeInline: boolean;

  constructor(options: IncrementalMarkdownRendererOptions = {}) {
    const napi = importNapiModuleSync();
    this.#native = new napi.IncrementalMarkdownRenderer(toNativeParserOptions(options));
    this.#renderPending = options.renderPending ?? true;
    this.#completeInline = options.completeInline ?? true;
  }

  append(
    chunk: string,
    options: IncrementalMarkdownRenderAppendOptions = {},
  ): IncrementalMarkdownRenderResult {
    return this.#native.append(chunk, {
      isFinal: options.final ?? false,
      renderPending: options.renderPending ?? this.#renderPending,
      completeInline: options.completeInline ?? this.#completeInline,
    });
  }

  finish(): IncrementalMarkdownRenderResult {
    return this.#native.finish();
  }

  reset(): void {
    this.#native.reset();
  }

  get committedHtml(): string {
    return this.#native.committedHtml;
  }

  get pendingMarkdown(): string {
    return this.#native.pendingMarkdown;
  }
}

export function createIncrementalMarkdownParser<TAst = unknown>(
  options?: IncrementalMarkdownParserOptions & IncrementalMarkdownParseAppendOptions,
): IncrementalMarkdownParser<TAst> {
  return new IncrementalMarkdownParser<TAst>(options);
}

export function createIncrementalMarkdownRenderer(
  options?: IncrementalMarkdownRendererOptions,
): IncrementalMarkdownRenderer {
  return new IncrementalMarkdownRenderer(options);
}

export async function* renderMarkdownStream(
  chunks: MarkdownChunkSource,
  options: IncrementalMarkdownRendererOptions = {},
): AsyncGenerator<IncrementalMarkdownRenderResult> {
  const renderer = createIncrementalMarkdownRenderer(options);

  for await (const chunk of chunks) {
    yield renderer.append(chunk);
  }

  yield renderer.finish();
}
