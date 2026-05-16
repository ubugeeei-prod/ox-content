import { createMarkdownEditor } from "./monaco.ts";

type ViewId = "preview" | "html" | "ast";

type ListItemNode = {
  type: "listItem";
  text: string;
  checked?: boolean;
};

type AstNode = {
  type: string;
  depth?: number;
  lang?: string;
  ordered?: boolean;
  text?: string;
  items?: ListItemNode[];
};

type ParseResult = {
  html: string;
  ast: string;
  renderMs: number;
};

type HighlightRule = {
  type:
    | "comment"
    | "function"
    | "keyword"
    | "number"
    | "operator"
    | "property"
    | "punctuation"
    | "string"
    | "tag"
    | "type"
    | "variable";
  pattern: RegExp;
};

type HighlightMode =
  | "plain"
  | "script"
  | "json"
  | "markup"
  | "style"
  | "shell"
  | "data"
  | "markdown";

const defaultMarkdown = `# Ox Content

Build docs with a parser that is designed to stay fast as projects grow.

## Why teams reach for it

- Arena-allocated parsing keeps memory churn low
- HTML rendering is bundled with the same Rust core
- The Vite plugin turns Markdown into a docs system quickly

## A tiny example

\`\`\`ts
import { oxContent } from "@ox-content/vite-plugin"

export default {
  plugins: [oxContent({ srcDir: "content", highlight: true })],
}
\`\`\`

> This browser demo uses a lightweight parser shim so you can iterate on content and inspect the shape of the output.

Visit the [repository](https://github.com/ubugeeei/ox-content) to see the full project.`;

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escapeAttribute(value: string): string {
  return escapeHtml(value).replace(/"/g, "&quot;");
}

const LANGUAGE_ALIASES: Record<string, HighlightMode> = {
  bash: "shell",
  c: "script",
  cpp: "script",
  css: "style",
  go: "script",
  htm: "markup",
  html: "markup",
  ini: "data",
  java: "script",
  js: "script",
  json: "json",
  jsonc: "json",
  jsx: "script",
  md: "markdown",
  mjs: "script",
  py: "script",
  python: "script",
  rs: "script",
  rust: "script",
  sass: "style",
  scss: "style",
  shell: "shell",
  sh: "shell",
  sql: "script",
  svg: "markup",
  toml: "data",
  ts: "script",
  tsx: "script",
  typescript: "script",
  xml: "markup",
  yaml: "data",
  yml: "data",
  zsh: "shell",
};

const LANGUAGE_LABELS: Record<string, string> = {
  ast: "AST",
  bash: "BASH",
  css: "CSS",
  html: "HTML",
  javascript: "JS",
  js: "JS",
  json: "JSON",
  jsx: "JSX",
  markdown: "MD",
  md: "MD",
  shell: "SH",
  sh: "SH",
  text: "",
  toml: "TOML",
  ts: "TS",
  tsx: "TSX",
  typescript: "TS",
  xml: "XML",
  yaml: "YAML",
  yml: "YAML",
};

const SCRIPT_RULES: HighlightRule[] = [
  {
    type: "string",
    pattern: /`(?:\\[\s\S]|[^`])*`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/,
  },
  {
    type: "comment",
    pattern: /\/\*[\s\S]*?\*\/|\/\/.*/,
  },
  {
    type: "keyword",
    pattern:
      /\b(?:as|async|await|break|case|catch|class|const|continue|crate|default|delete|do|else|enum|export|extends|false|finally|fn|for|from|function|if|implements|import|in|instanceof|interface|let|loop|match|mod|mut|new|null|package|private|protected|pub|public|return|self|static|struct|super|switch|this|throw|trait|true|try|type|typeof|undefined|use|var|while|yield)\b/,
  },
  {
    type: "type",
    pattern:
      /\b(?:Array|Boolean|Date|Error|Map|Number|Object|Promise|ReadonlyArray|RegExp|Set|String|symbol|unknown|never|void|any)\b|\b[A-Z][A-Za-z0-9_]*\b/,
  },
  {
    type: "property",
    pattern: /\b[A-Za-z_$][\w$]*(?=\s*:)/,
  },
  {
    type: "number",
    pattern: /\b(?:0[xX][\da-fA-F]+|0[bB][01]+|0[oO][0-7]+|\d+(?:\.\d+)?(?:e[+-]?\d+)?)\b/,
  },
  {
    type: "function",
    pattern: /\b[A-Za-z_$][\w$]*(?=\s*\()/,
  },
  {
    type: "operator",
    pattern: /=>|===|!==|==|!=|<=|>=|\+\+|--|&&|\|\||\??\.|[=+\-*/%<>!&|^~?:]+/,
  },
  {
    type: "punctuation",
    pattern: /[()[\]{}.,;]/,
  },
];

const JSON_RULES: HighlightRule[] = [
  {
    type: "property",
    pattern: /"(?:\\.|[^"\\])*"(?=\s*:)/,
  },
  {
    type: "string",
    pattern: /"(?:\\.|[^"\\])*"/,
  },
  {
    type: "keyword",
    pattern: /\b(?:true|false|null)\b/,
  },
  {
    type: "number",
    pattern: /\b(?:-?\d+(?:\.\d+)?(?:e[+-]?\d+)?)\b/,
  },
  {
    type: "punctuation",
    pattern: /[{}[\],:]/,
  },
];

const MARKUP_RULES: HighlightRule[] = [
  {
    type: "comment",
    pattern: /<!--[\s\S]*?-->/,
  },
  {
    type: "string",
    pattern: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/,
  },
  {
    type: "tag",
    pattern: /<\/?[A-Za-z][\w:-]*/,
  },
  {
    type: "property",
    pattern: /\b[A-Za-z_:][-A-Za-z0-9_:.]*(?==)/,
  },
  {
    type: "operator",
    pattern: /=/,
  },
  {
    type: "punctuation",
    pattern: /\/?>|</,
  },
];

const STYLE_RULES: HighlightRule[] = [
  {
    type: "comment",
    pattern: /\/\*[\s\S]*?\*\//,
  },
  {
    type: "string",
    pattern: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/,
  },
  {
    type: "keyword",
    pattern: /@[a-z-]+/,
  },
  {
    type: "property",
    pattern: /\b[a-z-]+(?=\s*:)/,
  },
  {
    type: "number",
    pattern: /-?(?:\d*\.)?\d+(?:px|rem|em|%|vh|vw|ms|s|deg|fr)?\b/,
  },
  {
    type: "function",
    pattern: /\b[a-z-]+(?=\()/,
  },
  {
    type: "type",
    pattern: /#[\da-fA-F]{3,8}\b|\b(?:rgba?|hsla?|url|linear-gradient|radial-gradient)\b/,
  },
  {
    type: "punctuation",
    pattern: /[{}():;.,]/,
  },
];

const SHELL_RULES: HighlightRule[] = [
  {
    type: "string",
    pattern: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/,
  },
  {
    type: "comment",
    pattern: /#.*/,
  },
  {
    type: "keyword",
    pattern: /\b(?:if|then|else|fi|for|do|done|case|esac|while|in|function|export|local|return)\b/,
  },
  {
    type: "function",
    pattern: /\b(?:pnpm|npm|node|npx|git|cargo|nix|vp|vite|cd|ls|cat|echo|cp|mv|rm|mkdir|touch)\b/,
  },
  {
    type: "variable",
    pattern: /\$\{[^}]+\}|\$[A-Za-z_][\w]*/,
  },
  {
    type: "number",
    pattern: /\b\d+\b/,
  },
  {
    type: "operator",
    pattern: /&&|\|\||[|><=&-]+/,
  },
  {
    type: "punctuation",
    pattern: /[()[\]{}]/,
  },
];

const DATA_RULES: HighlightRule[] = [
  {
    type: "string",
    pattern: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/,
  },
  {
    type: "comment",
    pattern: /#.*/,
  },
  {
    type: "property",
    pattern: /^[A-Za-z0-9_.-]+(?=\s*[:=])/,
  },
  {
    type: "keyword",
    pattern: /\b(?:true|false|null)\b/,
  },
  {
    type: "number",
    pattern: /\b(?:\d{4}-\d{2}-\d{2}|-?\d+(?:\.\d+)?)\b/,
  },
  {
    type: "punctuation",
    pattern: /[{}[\],:=]/,
  },
];

const MARKDOWN_RULES: HighlightRule[] = [
  {
    type: "comment",
    pattern: /<!--[\s\S]*?-->/,
  },
  {
    type: "keyword",
    pattern: /^#{1,6}\s.+$/,
  },
  {
    type: "operator",
    pattern: /^```[\w-]*$|^~~~[\w-]*$/,
  },
  {
    type: "property",
    pattern: /^(?:[-*+]|\d+\.)\s.+$/,
  },
  {
    type: "string",
    pattern: /\[[^\]]+\]\([^)]+\)|`[^`]+`|\*\*[^*]+\*\*|_[^_]+_/,
  },
];

const HIGHLIGHT_RULES: Record<Exclude<HighlightMode, "plain">, HighlightRule[]> = {
  data: DATA_RULES,
  json: JSON_RULES,
  markdown: MARKDOWN_RULES,
  markup: MARKUP_RULES,
  script: SCRIPT_RULES,
  shell: SHELL_RULES,
  style: STYLE_RULES,
};

function normalizeHighlightMode(language: string): HighlightMode {
  const normalized = language.trim().toLowerCase();

  if (!normalized || normalized === "plain" || normalized === "text") {
    return "plain";
  }

  return LANGUAGE_ALIASES[normalized] ?? "plain";
}

function getLanguageLabel(language: string): string {
  const normalized = language.trim().toLowerCase();

  if (!normalized || normalized === "plain" || normalized === "text") {
    return "";
  }

  return LANGUAGE_LABELS[normalized] ?? normalized.toUpperCase();
}

function renderHighlightedSource(source: string, language: string): string {
  const mode = normalizeHighlightMode(language);

  if (mode === "plain") {
    return escapeHtml(source);
  }

  const rules = HIGHLIGHT_RULES[mode];
  const pattern = new RegExp(
    rules.map((rule, index) => `(?<rule${index}>${rule.pattern.source})`).join("|"),
    "gm",
  );

  let rendered = "";
  let lastIndex = 0;

  for (const match of source.matchAll(pattern)) {
    const index = match.index ?? 0;
    const value = match[0];

    if (!value) {
      continue;
    }

    rendered += escapeHtml(source.slice(lastIndex, index));

    const tokenIndex = rules.findIndex(
      (_rule, ruleIndex) => match.groups?.[`rule${ruleIndex}`] !== undefined,
    );
    const tokenType = tokenIndex >= 0 ? rules[tokenIndex].type : "string";

    rendered += `<span class="token token-${tokenType}">${escapeHtml(value)}</span>`;
    lastIndex = index + value.length;
  }

  rendered += escapeHtml(source.slice(lastIndex));

  return rendered;
}

function setCodeFrameLanguage(element: HTMLElement, language: string): void {
  const label = getLanguageLabel(language);

  if (!label) {
    delete element.dataset.language;
    return;
  }

  element.dataset.language = label;
}

function highlightPreviewCodeBlocks(container: ParentNode): void {
  const blocks = container.querySelectorAll("pre > code");

  blocks.forEach((block) => {
    const languageClass = Array.from(block.classList).find((name) => name.startsWith("language-"));
    const language = languageClass ? languageClass.replace("language-", "") : "plain";

    if (block.parentElement instanceof HTMLPreElement) {
      setCodeFrameLanguage(block.parentElement, language);
    }

    block.innerHTML = renderHighlightedSource(block.textContent ?? "", language);
  });
}

function renderCodePane(element: HTMLPreElement, source: string, language: string): void {
  setCodeFrameLanguage(element, language);
  element.innerHTML = renderHighlightedSource(source, language);
}

function sanitizeHref(value: string): string {
  const href = value.trim();
  if (/^(https?:\/\/|mailto:|\/|#)/.test(href)) {
    return escapeAttribute(href);
  }
  return "#";
}

function renderInline(text: string): string {
  const codeTokens: string[] = [];
  let rendered = escapeHtml(text);

  rendered = rendered.replace(/`([^`]+)`/g, (_match, code) => {
    const token = `@@code-${codeTokens.length}@@`;
    codeTokens.push(`<code>${code}</code>`);
    return token;
  });

  rendered = rendered
    .replace(
      /\[([^\]]+)\]\(([^)]+)\)/g,
      (_match, label, href) =>
        `<a href="${sanitizeHref(href)}" target="_blank" rel="noreferrer">${label}</a>`,
    )
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/_([^_]+)_/g, "<em>$1</em>");

  return codeTokens.reduce(
    (value, token, index) => value.replace(`@@code-${index}@@`, token),
    rendered,
  );
}

function renderParagraph(lines: string[]): string {
  return lines
    .map((line) => renderInline(line.trim()))
    .join("<br />")
    .replace(/(<br \/>)+$/, "");
}

function parseMarkdown(source: string): ParseResult {
  const startedAt = performance.now();
  const htmlBlocks: string[] = [];
  const astNodes: AstNode[] = [];
  const lines = source.replace(/\r\n/g, "\n").split("\n");

  let paragraphLines: string[] = [];
  let quoteLines: string[] = [];
  let codeBlock: { lang: string; lines: string[] } | null = null;
  let listState: {
    ordered: boolean;
    items: Array<{ text: string; checked?: boolean }>;
  } | null = null;

  const flushParagraph = () => {
    const text = paragraphLines.join("\n").trim();
    if (!text) {
      paragraphLines = [];
      return;
    }

    htmlBlocks.push(`<p>${renderParagraph(paragraphLines)}</p>`);
    astNodes.push({ type: "paragraph", text });
    paragraphLines = [];
  };

  const flushQuote = () => {
    const text = quoteLines.join("\n").trim();
    if (!text) {
      quoteLines = [];
      return;
    }

    htmlBlocks.push(`<blockquote><p>${renderParagraph(quoteLines)}</p></blockquote>`);
    astNodes.push({ type: "blockquote", text });
    quoteLines = [];
  };

  const flushList = () => {
    if (!listState || listState.items.length === 0) {
      listState = null;
      return;
    }

    const tag = listState.ordered ? "ol" : "ul";
    const itemsHtml = listState.items
      .map((item) => {
        if (item.checked === undefined) {
          return `<li>${renderInline(item.text)}</li>`;
        }

        const stateClass = item.checked ? " checked" : "";
        const marker = item.checked ? "x" : " ";

        return `<li class="task-item"><span class="task-mark${stateClass}" aria-hidden="true">${marker}</span><span>${renderInline(item.text)}</span></li>`;
      })
      .join("");

    htmlBlocks.push(`<${tag}>${itemsHtml}</${tag}>`);
    astNodes.push({
      type: listState.ordered ? "orderedList" : "unorderedList",
      ordered: listState.ordered,
      items: listState.items.map((item) => ({
        type: "listItem",
        text: item.text,
        checked: item.checked,
      })),
    });
    listState = null;
  };

  const flushTextBlocks = () => {
    flushParagraph();
    flushQuote();
    flushList();
  };

  const flushCodeBlock = () => {
    if (!codeBlock) {
      return;
    }

    const language = codeBlock.lang || "plain";
    const value = codeBlock.lines.join("\n");
    htmlBlocks.push(
      `<pre><code class="language-${escapeAttribute(language)}">${escapeHtml(value)}</code></pre>`,
    );
    astNodes.push({
      type: "code",
      lang: language,
      text: value,
    });
    codeBlock = null;
  };

  for (const rawLine of lines) {
    const line = rawLine.replace(/\t/g, "  ");
    const fenceMatch = line.match(/^```([\w-]*)\s*$/);

    if (codeBlock) {
      if (fenceMatch) {
        flushCodeBlock();
      } else {
        codeBlock.lines.push(rawLine);
      }
      continue;
    }

    if (fenceMatch) {
      flushTextBlocks();
      codeBlock = { lang: fenceMatch[1] || "plain", lines: [] };
      continue;
    }

    if (/^\s*$/.test(line)) {
      flushTextBlocks();
      continue;
    }

    const headingMatch = line.match(/^(#{1,6})\s+(.*)$/);
    if (headingMatch) {
      flushTextBlocks();
      const depth = headingMatch[1].length;
      const text = headingMatch[2].trim();
      htmlBlocks.push(`<h${depth}>${renderInline(text)}</h${depth}>`);
      astNodes.push({ type: "heading", depth, text });
      continue;
    }

    if (/^---+$/.test(line.trim())) {
      flushTextBlocks();
      htmlBlocks.push("<hr />");
      astNodes.push({ type: "thematicBreak" });
      continue;
    }

    const quoteMatch = line.match(/^>\s?(.*)$/);
    if (quoteMatch) {
      flushParagraph();
      flushList();
      quoteLines.push(quoteMatch[1]);
      continue;
    }

    const taskMatch = line.match(/^[-*+]\s+\[( |x|X)\]\s+(.*)$/);
    if (taskMatch) {
      flushParagraph();
      flushQuote();
      if (!listState || listState.ordered) {
        listState = { ordered: false, items: [] };
      }
      listState.items.push({
        text: taskMatch[2],
        checked: taskMatch[1].toLowerCase() === "x",
      });
      continue;
    }

    const unorderedMatch = line.match(/^[-*+]\s+(.*)$/);
    if (unorderedMatch) {
      flushParagraph();
      flushQuote();
      if (!listState || listState.ordered) {
        listState = { ordered: false, items: [] };
      }
      listState.items.push({ text: unorderedMatch[1] });
      continue;
    }

    const orderedMatch = line.match(/^\d+\.\s+(.*)$/);
    if (orderedMatch) {
      flushParagraph();
      flushQuote();
      if (!listState || !listState.ordered) {
        listState = { ordered: true, items: [] };
      }
      listState.items.push({ text: orderedMatch[1] });
      continue;
    }

    paragraphLines.push(line);
  }

  flushTextBlocks();
  flushCodeBlock();

  return {
    html: htmlBlocks.join("\n\n"),
    ast: JSON.stringify(
      {
        type: "document",
        parser: "browser-shim",
        children: astNodes,
      },
      null,
      2,
    ),
    renderMs: performance.now() - startedAt,
  };
}

const markdownEditorHost = document.getElementById("markdown-editor") as HTMLDivElement;
const previewContent = document.getElementById("preview-content") as HTMLDivElement;
const htmlContent = document.getElementById("html-content") as HTMLPreElement;
const astContent = document.getElementById("ast-content") as HTMLPreElement;

const previewTab = document.getElementById("preview-tab") as HTMLButtonElement;
const htmlTab = document.getElementById("html-tab") as HTMLButtonElement;
const astTab = document.getElementById("ast-tab") as HTMLButtonElement;

const previewView = document.getElementById("preview-view") as HTMLDivElement;
const htmlView = document.getElementById("html-view") as HTMLDivElement;
const astView = document.getElementById("ast-view") as HTMLDivElement;

const resetButton = document.getElementById("reset-button") as HTMLButtonElement;
const copyMarkdownButton = document.getElementById("copy-markdown-button") as HTMLButtonElement;
const copyOutputButton = document.getElementById("copy-output-button") as HTMLButtonElement;

const tabs: Record<ViewId, HTMLButtonElement> = {
  preview: previewTab,
  html: htmlTab,
  ast: astTab,
};

const views: Record<ViewId, HTMLDivElement> = {
  preview: previewView,
  html: htmlView,
  ast: astView,
};

let activeView: ViewId = "preview";
let currentHtml = "";
let currentAst = "";
const buttonTimers = new WeakMap<HTMLButtonElement, number>();
const markdownEditor = createMarkdownEditor(markdownEditorHost, defaultMarkdown);

function updateCopyButtonLabel(): void {
  const label = activeView === "ast" ? "Copy AST" : "Copy HTML";
  copyOutputButton.dataset.label = label;
  copyOutputButton.textContent = label;
}

function switchTab(nextView: ViewId): void {
  activeView = nextView;

  (Object.keys(tabs) as ViewId[]).forEach((viewId) => {
    const isActive = viewId === nextView;
    tabs[viewId].classList.toggle("active", isActive);
    tabs[viewId].setAttribute("aria-selected", String(isActive));
    views[viewId].classList.toggle("active", isActive);
  });

  updateCopyButtonLabel();
}

function flashButtonLabel(button: HTMLButtonElement, label: string): void {
  const restoreLabel = button.dataset.label ?? button.textContent ?? "";
  const currentTimer = buttonTimers.get(button);

  if (currentTimer) {
    window.clearTimeout(currentTimer);
  }

  button.textContent = label;

  const timer = window.setTimeout(() => {
    button.textContent = restoreLabel;
  }, 1400);

  buttonTimers.set(button, timer);
}

async function copyText(
  button: HTMLButtonElement,
  value: string,
  successLabel: string,
): Promise<void> {
  try {
    await navigator.clipboard.writeText(value);
    flashButtonLabel(button, successLabel);
  } catch {
    flashButtonLabel(button, "Clipboard unavailable");
  }
}

function updatePreview(): void {
  const source = markdownEditor.getValue();
  const result = parseMarkdown(source);

  currentHtml = result.html;
  currentAst = result.ast;

  previewContent.innerHTML = result.html;
  highlightPreviewCodeBlocks(previewContent);
  renderCodePane(htmlContent, result.html, "html");
  renderCodePane(astContent, result.ast, "json");
  copyOutputButton.title = `Updated in ${result.renderMs.toFixed(2)} ms`;
}

markdownEditor.onDidChangeValue(updatePreview);

previewTab.addEventListener("click", () => switchTab("preview"));
htmlTab.addEventListener("click", () => switchTab("html"));
astTab.addEventListener("click", () => switchTab("ast"));

resetButton.addEventListener("click", () => {
  markdownEditor.setValue(defaultMarkdown);
  markdownEditor.focus();
  updatePreview();
});
copyMarkdownButton.addEventListener("click", () => {
  void copyText(copyMarkdownButton, markdownEditor.getValue(), "Copied");
});
copyOutputButton.addEventListener("click", () => {
  const output = activeView === "ast" ? currentAst : currentHtml;
  void copyText(copyOutputButton, output, "Copied");
});

switchTab("preview");
updatePreview();
markdownEditor.focus();

console.log(
  "%c Ox Content Playground ",
  "background: #d66a45; color: #fffaf2; font-weight: 700; padding: 4px 10px; border-radius: 999px;",
);
console.log("Browser demo ready");
