/**
 * Source Documentation Extraction and Generation
 *
 * This module provides comprehensive tools for extracting JSDoc/TSDoc comments
 * from TypeScript/JavaScript source files and automatically generating Markdown
 * documentation.
 *
 * ## Features
 *
 * - **Automatic Extraction**: Parses declarations and structured JSDoc metadata in Rust
 * - **Flexible Filtering**: Include/exclude patterns for selective documentation
 * - **Markdown Generation**: Converts extracted docs to organized Markdown files
 * - **Navigation Generation**: Auto-generates sidebar navigation metadata
 * - **GitHub Links**: Includes clickable links to source code on GitHub
 *
 * ## Supported JSDoc Tags
 *
 * - `@param {type} name - description` - Function parameter documentation
 * - `@returns {type} description` - Return value documentation
 * - `@example` - Code examples (multi-line blocks)
 * - `@private` - Mark item as private (excluded from docs if private=false)
 * - `@default value` - Default parameter value
 * - Custom tags are preserved in the `tags` field
 *
 * ## Usage Flow
 *
 * 1. Call `extractDocs()` to parse source files
 * 2. Call `generateMarkdown()` to create Markdown content
 * 3. Call `writeDocs()` to write files to output directory
 * 4. Generated nav.ts can be imported for sidebar navigation
 *
 * @example
 * ```typescript
 * import { extractDocs, generateMarkdown, writeDocs } from './docs';
 *
 * const docsOptions = {
 *   enabled: true,
 *   src: ['./src'],
 *   out: './docs/api',
 *   include: ['**\/*.ts'],
 *   exclude: ['**\/*.test.ts'],
 *   groupBy: 'file',
 *   githubUrl: 'https://github.com/user/project',
 * };
 *
 * const extracted = await extractDocs(['./src'], docsOptions);
 * const markdown = generateMarkdown(extracted, docsOptions);
 * await writeDocs(markdown, './docs/api', extracted, docsOptions);
 * ```
 */

import * as fs from "fs";
import * as path from "path";
import type {
  ResolvedDocsOptions,
  ExtractedDocs,
  DocEntry,
  ParamDoc,
  GeneratedDocsData,
} from "./types";
import { generateNavMetadata, generateNavCode } from "./nav-generator";
import { importNapiModule } from "./napi";

const DOCS_MANIFEST_FILE = ".ox-content-docs-manifest.json";
const DOCS_DATA_FILE = "docs.json";

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function entryAnchor(name: string): string {
  return name.toLowerCase();
}

function cleanSummaryText(text: string | undefined, maxLength: number = 120): string {
  if (!text) {
    return "";
  }

  const collapsed = text.replace(/\s+/g, " ").trim();
  if (collapsed.length <= maxLength) {
    return collapsed;
  }

  return `${collapsed.slice(0, maxLength - 1).trimEnd()}…`;
}

function renderInlineHtml(text: string): string {
  let html = "";
  let lastIndex = 0;
  const tokenPattern =
    /`([^`]+)`|\[([^\]]+)\]\(([^)]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|\*([^*]+)\*|_([^_]+)_/g;
  let match: RegExpExecArray | null;

  while ((match = tokenPattern.exec(text)) !== null) {
    html += escapeHtml(text.slice(lastIndex, match.index));

    if (match[1]) {
      html += `<code>${escapeHtml(match[1])}</code>`;
    } else if (match[2] && match[3]) {
      html += `<a href="${escapeHtml(match[3])}">${renderInlineHtml(match[2])}</a>`;
    } else if (match[4] || match[5]) {
      const strongText = match[4] ?? match[5] ?? "";
      html += `<strong>${renderInlineHtml(strongText)}</strong>`;
    } else if (match[6] || match[7]) {
      const emphasisText = match[6] ?? match[7] ?? "";
      html += `<em>${renderInlineHtml(emphasisText)}</em>`;
    }

    lastIndex = match.index + match[0].length;
  }

  html += escapeHtml(text.slice(lastIndex));
  return html.replace(/\n/g, "<br>");
}

function isFenceStart(line: string): RegExpExecArray | null {
  return /^```([\w-]+)?\s*$/.exec(line.trim());
}

function isHeading(line: string): RegExpExecArray | null {
  return /^(#{1,6})\s+(.*)$/.exec(line.trim());
}

function isOrderedListItem(line: string): RegExpExecArray | null {
  return /^\d+\.\s+(.*)$/.exec(line.trim());
}

function isUnorderedListItem(line: string): RegExpExecArray | null {
  return /^[-*+]\s+(.*)$/.exec(line.trim());
}

function isMarkdownBlockStart(line: string): boolean {
  return Boolean(
    isFenceStart(line) || isHeading(line) || isOrderedListItem(line) || isUnorderedListItem(line),
  );
}

function renderMarkdownBlocksHtml(text: string): string {
  const lines = text.split(/\r?\n/);
  const blocks: string[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];
    const trimmed = line.trim();

    if (!trimmed) {
      index++;
      continue;
    }

    const fenceMatch = isFenceStart(line);
    if (fenceMatch) {
      const language = fenceMatch[1] || "text";
      const codeLines: string[] = [];
      index++;

      while (index < lines.length && !lines[index].trim().startsWith("```")) {
        codeLines.push(lines[index]);
        index++;
      }

      if (index < lines.length) {
        index++;
      }

      blocks.push(renderCodeBlockHtml(codeLines.join("\n"), language));
      continue;
    }

    const headingMatch = isHeading(line);
    if (headingMatch) {
      const level = Math.min(headingMatch[1].length, 6);
      blocks.push(`<h${level}>${renderInlineHtml(headingMatch[2].trim())}</h${level}>`);
      index++;
      continue;
    }

    const orderedMatch = isOrderedListItem(line);
    if (orderedMatch) {
      const items: string[] = [];

      while (index < lines.length) {
        const currentLine = lines[index];
        const currentMatch = isOrderedListItem(currentLine);
        if (!currentMatch) {
          break;
        }

        const itemLines = [currentMatch[1].trim()];
        index++;

        while (index < lines.length) {
          const continuation = lines[index];
          const continuationTrimmed = continuation.trim();

          if (
            !continuationTrimmed ||
            isMarkdownBlockStart(continuation) ||
            /^ {0,1}\d+\.\s+/.test(continuationTrimmed)
          ) {
            break;
          }

          itemLines.push(continuationTrimmed);
          index++;
        }

        items.push(`<li>${renderInlineHtml(itemLines.join(" "))}</li>`);

        if (index < lines.length && !lines[index].trim()) {
          break;
        }
      }

      blocks.push(`<ol>\n${items.join("\n")}\n</ol>`);
      continue;
    }

    const unorderedMatch = isUnorderedListItem(line);
    if (unorderedMatch) {
      const items: string[] = [];

      while (index < lines.length) {
        const currentLine = lines[index];
        const currentMatch = isUnorderedListItem(currentLine);
        if (!currentMatch) {
          break;
        }

        const itemLines = [currentMatch[1].trim()];
        index++;

        while (index < lines.length) {
          const continuation = lines[index];
          const continuationTrimmed = continuation.trim();

          if (
            !continuationTrimmed ||
            isMarkdownBlockStart(continuation) ||
            /^[-*+]\s+/.test(continuationTrimmed)
          ) {
            break;
          }

          itemLines.push(continuationTrimmed);
          index++;
        }

        items.push(`<li>${renderInlineHtml(itemLines.join(" "))}</li>`);

        if (index < lines.length && !lines[index].trim()) {
          break;
        }
      }

      blocks.push(`<ul>\n${items.join("\n")}\n</ul>`);
      continue;
    }

    const paragraphLines = [trimmed];
    index++;

    while (index < lines.length) {
      const nextLine = lines[index];
      const nextTrimmed = nextLine.trim();

      if (!nextTrimmed || isMarkdownBlockStart(nextLine)) {
        break;
      }

      paragraphLines.push(nextTrimmed);
      index++;
    }

    blocks.push(`<p>${renderInlineHtml(paragraphLines.join(" "))}</p>`);
  }

  return `<div class="ox-api-entry__prose">\n${blocks.join("\n")}\n</div>`;
}

function renderCodeBlockHtml(code: string, language: string = "typescript"): string {
  return `<pre><code class="language-${language}">${escapeHtml(code)}</code></pre>`;
}

function renderHighlightedInlineCodeHtml(
  code: string,
  className: string,
  language: string = "typescript",
): string {
  return `<code class="${escapeHtml(className)} language-${language}">${escapeHtml(code)}</code>`;
}

function renderDetailsControlsHtml(targetSelector: ".ox-api-entry" | ".ox-api-module"): string {
  return `<div class="ox-api-controls" data-ox-api-target="${targetSelector}" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>`;
}

function normalizeDocFilePath(filePath: string): string {
  const normalized = filePath.replace(/\\/g, "/");
  const match = normalized.match(/(?:^|\/)((?:npm|packages|crates|src)\/.+)$/);
  return match?.[1] ?? normalized.replace(/^\/+/, "");
}

function buildDocsData(docs: ExtractedDocs[]): GeneratedDocsData {
  return {
    version: 1,
    generatedAt: new Date().toISOString(),
    modules: docs.map((doc) => ({
      ...doc,
      file: normalizeDocFilePath(doc.file),
      entries: doc.entries.map((entry) => ({
        ...entry,
        file: normalizeDocFilePath(entry.file),
      })),
    })),
  };
}
interface NapiDocTag {
  tag: string;
  value: string;
}

interface NapiDocParam {
  name: string;
  typeAnnotation?: string;
  optional: boolean;
  defaultValue?: string;
  description?: string;
}

interface NapiDocItem {
  name: string;
  kind: string;
  doc?: string;
  jsdoc?: string;
  sourcePath: string;
  line: number;
  endLine: number;
  exported: boolean;
  signature?: string;
  params: NapiDocParam[];
  returnType?: string;
  returnDescription?: string;
  examples: string[];
  tags: NapiDocTag[];
}

/**
 * Extracts JSDoc documentation from source files in specified directories.
 *
 * This function recursively searches directories for source files matching
 * the include/exclude patterns, then extracts all documented items (functions,
 * classes, interfaces, types) from those files.
 *
 * ## Process
 *
 * 1. **File Discovery**: Recursively walks directories, applying filters
 * 2. **File Reading**: Loads each matching file's content
 * 3. **JSDoc Extraction**: Uses Rust-side JSDoc metadata from @ox-content/napi
 * 4. **Declaration Matching**: Pairs JSDoc comments with source declarations
 * 5. **Result Collection**: Aggregates extracted documentation by file
 *
 * ## Include/Exclude Patterns
 *
 * Patterns support:
 * - `**` - Match any directory structure
 * - `*` - Match any filename
 * - Standard glob patterns (e.g., `**\/*.test.ts`)
 *
 * ## Performance Considerations
 *
 * - Uses filesystem I/O which can be slow for large codebases
 * - Consider using more specific include patterns to reduce file scanning
 * - Results are not cached; call once per build/dev session
 *
 * @param srcDirs - Array of source directory paths to scan
 * @param options - Documentation extraction options (filters, grouping, etc.)
 *
 * @returns Promise resolving to array of extracted documentation by file.
 *          Each ExtractedDocs object contains file path and array of DocEntry items.
 *
 * @example
 * ```typescript
 * const docs = await extractDocs(
 *   ['./packages/vite-plugin/src'],
 *   {
 *     enabled: true,
 *     src: [],
 *     out: 'docs',
 *     include: ['**\/*.ts'],
 *     exclude: ['**\/*.test.ts', '**\/*.spec.ts'],
 *     format: 'markdown',
 *     private: false,
 *     toc: true,
 *     groupBy: 'file',
 *     generateNav: true,
 *   }
 * );
 *
 * // Returns:
 * // [
 * //   {
 * //     file: '/path/to/transform.ts',
 * //     entries: [
 * //       { name: 'transformMarkdown', kind: 'function', ... },
 * //       { name: 'loadNapiBindings', kind: 'function', ... },
 * //     ]
 * //   },
 * //   ...
 * // ]
 * ```
 */
export async function extractDocs(
  srcDirs: string[],
  options: ResolvedDocsOptions,
): Promise<ExtractedDocs[]> {
  const napi = await importNapiModule();
  const extractFileDocs = (
    napi as { extractFileDocs?: (filePath: string, includePrivate?: boolean) => NapiDocItem[] }
  ).extractFileDocs;

  if (!extractFileDocs) {
    throw new Error("[ox-content] extractFileDocs is not available from @ox-content/napi.");
  }

  const results: ExtractedDocs[] = [];

  for (const srcDir of srcDirs) {
    const files = await findFiles(srcDir, options);

    for (const file of files) {
      const entries = extractFileDocs(file, options.private)
        .map(parseNapiDocItem)
        .filter((entry): entry is DocEntry => Boolean(entry));

      if (entries.length > 0) {
        results.push({ file, entries });
      }
    }
  }

  return results;
}

/**
 * Recursively finds all source files matching include/exclude patterns.
 *
 * @internal
 */
async function findFiles(dir: string, options: ResolvedDocsOptions): Promise<string[]> {
  const files: string[] = [];

  async function walk(currentDir: string) {
    let entries;
    try {
      entries = await fs.promises.readdir(currentDir, { withFileTypes: true });
    } catch {
      return;
    }

    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);

      if (entry.isDirectory()) {
        if (!isExcluded(fullPath, options.exclude)) {
          await walk(fullPath);
        }
      } else if (entry.isFile()) {
        if (isIncluded(fullPath, options.include) && !isExcluded(fullPath, options.exclude)) {
          files.push(fullPath);
        }
      }
    }
  }

  await walk(dir);
  return files;
}

function isIncluded(file: string, patterns: string[]): boolean {
  return patterns.some((pattern) => {
    if (pattern.includes("**")) {
      const ext = pattern.split(".").pop();
      return file.endsWith(`.${ext}`);
    }
    return file.endsWith(pattern.replace("*", ""));
  });
}

function isExcluded(file: string, patterns: string[]): boolean {
  return patterns.some((pattern) => {
    if (pattern.includes("node_modules")) {
      return file.includes("node_modules");
    }
    if (pattern.includes(".test.") || pattern.includes(".spec.")) {
      return file.includes(".test.") || file.includes(".spec.");
    }
    return false;
  });
}

function parseNapiDocItem(item: NapiDocItem): DocEntry | null {
  const kind = normalizeNapiKind(item.kind);
  if (!kind) {
    return null;
  }

  const params = item.params.map((param) => ({
    name: param.name,
    type: param.typeAnnotation ?? "unknown",
    description: param.description ?? "",
    optional: param.optional || undefined,
    default: param.defaultValue,
  }));
  const returns = item.returnType
    ? {
        type: item.returnType,
        description: item.returnDescription ?? "",
      }
    : undefined;
  const tags: Record<string, string> = {};
  let isPrivate = false;

  for (const tag of item.tags) {
    if (
      tag.tag === "param" ||
      tag.tag === "returns" ||
      tag.tag === "return" ||
      tag.tag === "example"
    ) {
      continue;
    }
    if (tag.tag === "private") {
      isPrivate = true;
      continue;
    }
    if (!tags[tag.tag]) {
      tags[tag.tag] = tag.value;
    }
  }

  return {
    name: item.name,
    kind,
    description: item.doc ?? "",
    params: params.length > 0 ? params : undefined,
    returns,
    examples: item.examples.length > 0 ? item.examples : undefined,
    tags: Object.keys(tags).length > 0 ? tags : undefined,
    private: isPrivate,
    file: item.sourcePath,
    line: item.line,
    endLine: item.endLine,
    signature: item.signature,
  };
}

function normalizeNapiKind(kind: string): DocEntry["kind"] | null {
  switch (kind) {
    case "function":
    case "class":
    case "interface":
    case "type":
    case "variable":
    case "module":
      return kind;
    case "enum":
      return "type";
    default:
      return null;
  }
}

/**
 * Generates Markdown documentation from extracted docs.
 */
export function generateMarkdown(
  docs: ExtractedDocs[],
  options: ResolvedDocsOptions,
): Record<string, string> {
  const result: Record<string, string> = {};
  const sortedDocs = sortExtractedDocs(docs);
  const symbolMap = buildSymbolMap(sortedDocs);

  if (options.groupBy === "file") {
    const docToFile = new Map<ExtractedDocs, string>();

    for (const doc of sortedDocs) {
      let fileName = path.basename(doc.file, path.extname(doc.file));
      // Avoid conflict with the main index.md
      if (fileName === "index") {
        fileName = "index-module";
      }
      docToFile.set(doc, fileName);

      const markdown = generateFileMarkdown(doc, options, fileName, symbolMap);
      result[`${fileName}.md`] = markdown;
    }

    result["index.md"] = generateIndex(sortedDocs, docToFile);
  } else {
    const byKind = new Map<string, DocEntry[]>();

    for (const doc of sortedDocs) {
      for (const entry of doc.entries) {
        const existing = byKind.get(entry.kind) || [];
        existing.push(entry);
        byKind.set(entry.kind, existing);
      }
    }

    for (const entries of byKind.values()) {
      entries.sort(compareEntriesByName);
    }

    for (const [kind, entries] of [...byKind.entries()].sort(([a], [b]) => compareStrings(a, b))) {
      result[`${kind}s.md`] = generateCategoryMarkdown(kind, entries, options, symbolMap);
    }

    result["index.md"] = generateCategoryIndex(byKind);
  }

  return result;
}

function compareStrings(a: string, b: string): number {
  return a.localeCompare(b, "en", {
    numeric: true,
    sensitivity: "base",
  });
}

function compareEntriesByName(a: DocEntry, b: DocEntry): number {
  return compareStrings(a.name, b.name);
}

function sortExtractedDocs(docs: ExtractedDocs[]): ExtractedDocs[] {
  return [...docs]
    .map((doc) => ({
      ...doc,
      entries: [...doc.entries].sort(compareEntriesByName),
    }))
    .sort((a, b) => compareStrings(path.basename(a.file), path.basename(b.file)));
}

function generateFileMarkdown(
  doc: ExtractedDocs,
  options: ResolvedDocsOptions,
  currentFileName: string,
  symbolMap: Map<string, SymbolLocation>,
): string {
  const displayName = path.basename(doc.file);
  let md = `# ${displayName}\n\n`;

  // Add source link if githubUrl is provided
  if (options.githubUrl) {
    const sourceLink = generateSourceLink(doc.file, options.githubUrl);
    if (sourceLink) {
      md += sourceLink + "\n\n";
    }
  }

  md += `> ${doc.entries.length} documented symbol${doc.entries.length === 1 ? "" : "s"}. `;
  md +=
    "Read the signatures first, then expand each item for parameters, return types, and examples.\n\n";

  md += "## Reference\n\n";
  if (doc.entries.length > 1) {
    md += renderDetailsControlsHtml(".ox-api-entry") + "\n\n";
  }

  for (const entry of doc.entries) {
    md += generateEntryMarkdown(entry, options, currentFileName, symbolMap);
  }

  return md;
}

function normalizeSignature(signature: string | undefined): string | undefined {
  if (!signature) {
    return undefined;
  }

  return signature
    .replace(/\s+/g, " ")
    .replace(/^export\s+/, "")
    .replace(/^declare\s+/, "")
    .replace(/^abstract\s+/, "")
    .replace(/^async\s+function\s+/, "")
    .replace(/^function\s+/, "")
    .replace(/^class\s+/, "")
    .replace(/^interface\s+/, "")
    .replace(/^type\s+/, "")
    .trim();
}

function formatKindLabel(kind: string): string {
  switch (kind) {
    case "function":
      return "fn";
    case "interface":
      return "interface";
    case "class":
      return "class";
    case "type":
      return "type";
    case "const":
      return "const";
    default:
      return kind;
  }
}

function renderOverviewLine(entry: DocEntry, href: string): string {
  const signature = normalizeSignature(entry.signature);
  const summary = cleanSummaryText(entry.description, 88);
  const parts = [`- [\`${entry.name}\`](${href})`, `\`${entry.kind}\``];

  if (signature) {
    parts.push(`\`${signature}\``);
  }

  if (summary) {
    parts.push(`- ${summary}`);
  }

  return `${parts.join(" ")}\n`;
}

function renderOverviewHtmlItem(entry: DocEntry, href: string): string {
  const signature = normalizeSignature(entry.signature);
  const summary = cleanSummaryText(entry.description, 88);
  const heading = signature
    ? `<a href="${escapeHtml(href)}" class="ox-api-module__link">${renderHighlightedInlineCodeHtml(signature, "ox-api-module__signature ox-api-module__signature--highlighted")}</a>`
    : `<a href="${escapeHtml(href)}" class="ox-api-module__link"><code class="ox-api-module__name">${escapeHtml(entry.name)}</code></a>`;

  return `<li><span class="ox-api-module__kind">${escapeHtml(formatKindLabel(entry.kind))}</span><div class="ox-api-module__item">${heading}${summary ? `<span class="ox-api-module__summary">${renderInlineHtml(summary)}</span>` : ""}</div></li>`;
}

function renderParamsListHtml(params: ParamDoc[]): string {
  const rows = params
    .map((param) => {
      const flags = [
        param.optional ? "optional" : "",
        param.default ? `default: ${param.default}` : "",
      ].filter(Boolean);
      const description = [param.description, flags.join(" · ")].filter(Boolean).join(" — ");

      return `<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">${escapeHtml(param.name)}</code>
    <code class="ox-api-entry__param-type">${escapeHtml(param.type)}</code>
  </div>
  ${description ? `<p class="ox-api-entry__param-description">${renderInlineHtml(description)}</p>` : ""}
</li>`;
    })
    .join("\n");

  return `<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
${rows}
</ul>
</div>`;
}

function renderTagListHtml(tags: Record<string, string>): string {
  const items = Object.entries(tags)
    .map(
      ([tag, value]) =>
        `<li><span class="ox-api-entry__tag-name">@${escapeHtml(tag)}</span><span class="ox-api-entry__tag-value">${renderInlineHtml(value)}</span></li>`,
    )
    .join("");

  return `<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags">${items}</ul>
</div>`;
}

function generateEntryMarkdown(
  entry: DocEntry,
  options?: ResolvedDocsOptions,
  currentFileName?: string,
  symbolMap?: Map<string, SymbolLocation>,
): string {
  const processedDescription =
    entry.description && currentFileName && symbolMap
      ? convertSymbolLinks(entry.description, currentFileName, symbolMap)
      : entry.description;
  const summarySignature = normalizeSignature(entry.signature);
  const sourceHref = options?.githubUrl
    ? generateSourceHref(entry.file, options.githubUrl, entry.line, entry.endLine)
    : undefined;

  let body = "";

  if (processedDescription) {
    body += renderMarkdownBlocksHtml(processedDescription) + "\n";
  }

  if (sourceHref) {
    body += `<p class="ox-api-entry__source"><a href="${escapeHtml(sourceHref)}">View source</a></p>\n`;
  }

  if (entry.params && entry.params.length > 0) {
    body += renderParamsListHtml(entry.params) + "\n";
  }

  if (entry.returns) {
    body += `<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">${escapeHtml(entry.returns.type)}</code>
  ${entry.returns.description ? `<p class="ox-api-entry__return-description">${renderInlineHtml(entry.returns.description)}</p>` : ""}
</div>
</div>\n`;
  }

  if (entry.examples && entry.examples.length > 0) {
    const examplesHtml = entry.examples
      .map((example) => example.replace(/^```\w*\n?/, "").replace(/\n?```$/, ""))
      .map((example) => renderCodeBlockHtml(example, "ts"))
      .join("\n");

    body += `<div class="ox-api-entry__section ox-api-entry__section--examples">\n<h4>Examples</h4>\n${examplesHtml}\n</div>\n`;
  }

  if (entry.tags && Object.keys(entry.tags).length > 0) {
    body += renderTagListHtml(entry.tags) + "\n";
  }

  const summaryDescription = cleanSummaryText(processedDescription, summarySignature ? 80 : 120);
  const summaryHeading = summarySignature
    ? renderHighlightedInlineCodeHtml(
        summarySignature,
        "ox-api-entry__signature ox-api-entry__signature--highlighted",
      )
    : `<code class="ox-api-entry__name">${escapeHtml(entry.name)}</code>`;
  const summaryParts = [
    `<span class="ox-api-entry__kind">${escapeHtml(formatKindLabel(entry.kind))}</span>`,
    `<span class="ox-api-entry__summary-main">${summaryHeading}${summaryDescription ? `<span class="ox-api-entry__description">${renderInlineHtml(summaryDescription)}</span>` : ""}</span>`,
  ];

  return `<details id="${entryAnchor(entry.name)}" class="ox-api-entry">
  <summary>${summaryParts.join("")}</summary>
  <div class="ox-api-entry__body">
${body.trim()}
  </div>
</details>

`;
}

function generateIndex(docs: ExtractedDocs[], docToFile?: Map<ExtractedDocs, string>): string {
  let md = "# API Documentation\n\n";
  md += "Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n";
  md +=
    "> Use search scopes like `@api transform` to limit results to the generated API reference.\n\n";

  md += "## Modules\n\n";
  if (docs.length > 1) {
    md += renderDetailsControlsHtml(".ox-api-module") + "\n\n";
  }

  for (const doc of docs) {
    const displayName = path.basename(doc.file, path.extname(doc.file));
    let fileName = displayName;

    if (docToFile && docToFile.has(doc)) {
      fileName = docToFile.get(doc)!;
    } else if (fileName === "index") {
      fileName = "index-module";
    }

    const countLabel = `${doc.entries.length} symbol${doc.entries.length === 1 ? "" : "s"}`;
    md += `<details class="ox-api-module">
  <summary>
    <span class="ox-api-module__title"><a href="./${fileName}.md">${escapeHtml(displayName)}</a></span>
    <span class="ox-api-module__count">${countLabel}</span>
  </summary>
  <div class="ox-api-module__body">
    <ul class="ox-api-module__list">
`;

    for (const entry of doc.entries) {
      md += `      ${renderOverviewHtmlItem(entry, `./${fileName}.md#${entryAnchor(entry.name)}`)}\n`;
    }

    md += `    </ul>
  </div>
</details>

`;
  }

  return md;
}

function generateCategoryMarkdown(
  kind: string,
  entries: DocEntry[],
  options: ResolvedDocsOptions,
  symbolMap: Map<string, SymbolLocation>,
): string {
  const categoryFileName = `${kind}s`;
  let md = `# ${kind.charAt(0).toUpperCase() + kind.slice(1)}s\n\n`;
  md += `> ${entries.length} documented ${kind}${entries.length === 1 ? "" : "s"} collected across modules.\n\n`;

  md += "## Overview\n\n";
  for (const entry of entries) {
    md += renderOverviewLine(entry, `#${entryAnchor(entry.name)}`);
  }
  md += "\n## Reference\n\n";
  if (entries.length > 1) {
    md += renderDetailsControlsHtml(".ox-api-entry") + "\n\n";
  }

  for (const entry of entries) {
    md += generateEntryMarkdown(entry, options, categoryFileName, symbolMap);
  }

  return md;
}

function generateCategoryIndex(byKind: Map<string, DocEntry[]>): string {
  let md = "# API Documentation\n\n";
  md += "Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n";

  for (const [kind, entries] of [...byKind.entries()].sort(([a], [b]) => compareStrings(a, b))) {
    const kindTitle = kind.charAt(0).toUpperCase() + kind.slice(1) + "s";
    md += `## [${kindTitle}](./${kind}s.md)\n\n`;
    md += `> ${entries.length} item${entries.length === 1 ? "" : "s"}.\n\n`;

    for (const entry of entries) {
      md += renderOverviewLine(entry, `./${kind}s.md#${entryAnchor(entry.name)}`);
    }
    md += "\n";
  }

  return md;
}

/**
 * Symbol location info for cross-file linking.
 */
interface SymbolLocation {
  name: string;
  file: string;
  fileName: string;
}

/**
 * Converts symbol links [SymbolName] to markdown links.
 *
 * Processes description text to convert cargo-docs-style symbol references
 * `[SymbolName]` into clickable markdown links pointing to the appropriate
 * documentation page.
 *
 * ## Examples
 *
 * Input: "See [transformMarkdown] for usage" (same file)
 * Output: "See [transformMarkdown](#transformmarkdown) for usage"
 *
 * Input: "Uses [NavItem] interface" (different file: types.ts)
 * Output: "Uses [NavItem](./types.md#navitem) interface"
 *
 * @param text - Description text containing symbol references
 * @param currentFileName - Current file name (without extension) for same-file detection
 * @param symbolMap - Map of symbol names to their file locations
 * @returns Text with symbol references converted to markdown links
 *
 * @internal
 */
function convertSymbolLinks(
  text: string,
  currentFileName: string,
  symbolMap: Map<string, SymbolLocation>,
): string {
  // Match [SymbolName] pattern where SymbolName starts with uppercase or underscore
  // Negative lookahead (?!\() ensures we don't match [Name] that's already part of [Name](url)
  return text.replace(/\[([A-Z_]\w*)\](?!\()/g, (match, symbolName) => {
    const location = symbolMap.get(symbolName);
    if (!location) {
      // Symbol not found, keep original text
      return match;
    }

    if (location.fileName === currentFileName) {
      // Same file - use anchor only
      return `[${symbolName}](#${symbolName.toLowerCase()})`;
    } else {
      // Different file - use cross-file link
      return `[${symbolName}](./${location.fileName}.md#${symbolName.toLowerCase()})`;
    }
  });
}

/**
 * Builds a map of all symbols to their file locations.
 */
function buildSymbolMap(docs: ExtractedDocs[]): Map<string, SymbolLocation> {
  const map = new Map<string, SymbolLocation>();

  for (const doc of docs) {
    let fileName = path.basename(doc.file, path.extname(doc.file));
    if (fileName === "index") {
      fileName = "index-module";
    }

    for (const entry of doc.entries) {
      map.set(entry.name, {
        name: entry.name,
        file: doc.file,
        fileName,
      });
    }
  }

  return map;
}

/**
 * Writes generated documentation to the output directory.
 */
export async function writeDocs(
  docs: Record<string, string>,
  outDir: string,
  extractedDocs?: ExtractedDocs[],
  options?: ResolvedDocsOptions,
): Promise<void> {
  await fs.promises.mkdir(outDir, { recursive: true });

  const generatedFiles = new Set(Object.keys(docs));
  if (extractedDocs && options?.generateNav && options.groupBy === "file") {
    generatedFiles.add("nav.ts");
  }
  if (extractedDocs) {
    generatedFiles.add(DOCS_DATA_FILE);
  }

  const manifestPath = path.join(outDir, DOCS_MANIFEST_FILE);
  let previousFiles: string[] = [];

  try {
    previousFiles = JSON.parse(await fs.promises.readFile(manifestPath, "utf-8")) as string[];
  } catch {
    previousFiles = [];
  }

  for (const staleFile of previousFiles) {
    if (generatedFiles.has(staleFile)) {
      continue;
    }

    await fs.promises.rm(path.join(outDir, staleFile), { force: true });
  }

  for (const [fileName, content] of Object.entries(docs)) {
    const filePath = path.join(outDir, fileName);
    await fs.promises.writeFile(filePath, content, "utf-8");
  }

  // Generate and write navigation metadata if enabled
  if (extractedDocs && options?.generateNav && options.groupBy === "file") {
    const navItems = generateNavMetadata(extractedDocs, "/api");
    const navCode = generateNavCode(navItems, "apiNav");
    const navFilePath = path.join(outDir, "nav.ts");
    await fs.promises.writeFile(navFilePath, navCode, "utf-8");
  }

  if (extractedDocs) {
    await fs.promises.writeFile(
      path.join(outDir, DOCS_DATA_FILE),
      JSON.stringify(buildDocsData(extractedDocs), null, 2),
      "utf-8",
    );
  }

  await fs.promises.writeFile(
    manifestPath,
    JSON.stringify([...generatedFiles].sort(), null, 2),
    "utf-8",
  );
}

/**
 * Resolves docs options with defaults.
 */
/**
 * Generates a GitHub source link for a file and optional line range.
 *
 * @param filePath - Full path to the source file
 * @param githubUrl - Base GitHub repository URL
 * @param lineNumber - Optional start line number to link to
 * @param endLineNumber - Optional end line number to link to
 * @returns Absolute GitHub URL to source code
 */
function generateSourceHref(
  filePath: string,
  githubUrl: string,
  lineNumber?: number,
  endLineNumber?: number,
): string {
  const relativePath = normalizeDocFilePath(filePath);

  const fragment = lineNumber
    ? endLineNumber && endLineNumber > lineNumber
      ? `#L${lineNumber}-L${endLineNumber}`
      : `#L${lineNumber}`
    : "";
  return `${githubUrl}/blob/main/${relativePath}${fragment}`;
}

function generateSourceLink(
  filePath: string,
  githubUrl: string,
  lineNumber?: number,
  endLineNumber?: number,
): string {
  return `**[Source](${generateSourceHref(filePath, githubUrl, lineNumber, endLineNumber)})**`;
}

export function resolveDocsOptions(options: false): false;
export function resolveDocsOptions(options?: import("./types").DocsOptions): ResolvedDocsOptions;
export function resolveDocsOptions(
  options: import("./types").DocsOptions | false | undefined,
): ResolvedDocsOptions | false;
export function resolveDocsOptions(
  options: import("./types").DocsOptions | false | undefined,
): ResolvedDocsOptions | false {
  if (options === false) {
    return false;
  }

  const opts = options || {};

  return {
    enabled: opts.enabled ?? true,
    src: opts.src ?? ["./src"],
    out: opts.out ?? "docs/api",
    include: opts.include ?? ["**/*.ts", "**/*.tsx"],
    exclude: opts.exclude ?? ["**/*.test.*", "**/*.spec.*", "node_modules"],
    format: opts.format ?? "markdown",
    private: opts.private ?? false,
    toc: false,
    groupBy: opts.groupBy ?? "file",
    githubUrl: opts.githubUrl,
    generateNav: opts.generateNav ?? true,
  };
}
