// monaco-editor 0.56 ships an `exports` map that only exposes the ESM tree
// (`monaco-editor/<path>` -> `esm/vs/<path>.js`), so the legacy `min/` and
// `esm/vs/` deep paths no longer resolve. The ESM entry point imports its own
// stylesheets and registers every language definition, so the separate
// `editor.main.css` and markdown contribution imports are gone.
import * as monaco from "monaco-editor";

import editorWorker from "monaco-editor/editor/editor.worker?worker";
import cssWorker from "monaco-editor/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/language/html/html.worker?worker";
import jsonWorker from "monaco-editor/language/json/json.worker?worker";
import tsWorker from "monaco-editor/language/typescript/ts.worker?worker";

type MonacoEnvironmentWindow = typeof globalThis & {
  MonacoEnvironment?: {
    getWorker: (_moduleId: string, label: string) => Worker;
  };
};

type MarkdownEditor = {
  focus: () => void;
  getValue: () => string;
  onDidChangeValue: (listener: () => void) => monaco.IDisposable;
  setValue: (value: string) => void;
};

type SnippetDefinition = {
  description: string;
  insertText: string;
  label: string;
};

const snippetDefinitions: SnippetDefinition[] = [
  {
    label: "h1",
    description: "Page heading",
    insertText: "# ${1:Title}\n\n$0",
  },
  {
    label: "h2",
    description: "Section heading",
    insertText: "## ${1:Section}\n\n$0",
  },
  {
    label: "quote",
    description: "Blockquote",
    insertText: "> ${1:Quoted text}\n\n$0",
  },
  {
    label: "list",
    description: "Bullet list",
    insertText: "- ${1:First item}\n- ${2:Second item}\n$0",
  },
  {
    label: "task",
    description: "Task list",
    insertText: "- [ ] ${1:Open item}\n- [x] ${2:Done item}\n$0",
  },
  {
    label: "link",
    description: "Markdown link",
    insertText: "[${1:label}](${2:https://example.com})$0",
  },
  {
    label: "image",
    description: "Markdown image",
    insertText: "![${1:alt text}](${2:/path/to/image.png})$0",
  },
  {
    label: "code",
    description: "TypeScript code fence",
    insertText: "```ts\n${1:const value = true}\n```\n$0",
  },
  {
    label: "table",
    description: "Simple table",
    insertText: "| ${1:Column} | ${2:Column} |\n| --- | --- |\n| ${3:Value} | ${4:Value} |\n$0",
  },
];

let themeRegistered = false;
let completionRegistered = false;

const monacoEnvironment = globalThis as MonacoEnvironmentWindow;

monacoEnvironment.MonacoEnvironment = {
  getWorker(_moduleId: string, label: string): Worker {
    switch (label) {
      case "css":
      case "less":
      case "scss":
        return new cssWorker();
      case "handlebars":
      case "html":
      case "razor":
        return new htmlWorker();
      case "javascript":
      case "typescript":
        return new tsWorker();
      case "json":
        return new jsonWorker();
      default:
        return new editorWorker();
    }
  },
};

function ensureTheme(): void {
  if (themeRegistered) {
    return;
  }

  monaco.editor.defineTheme("ox-playground", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "7F98A3" },
      { token: "keyword", foreground: "FFB36C" },
      { token: "string", foreground: "B8DF8B" },
      { token: "number", foreground: "FFCF70" },
      { token: "type", foreground: "B59DFF" },
      { token: "tag", foreground: "8FCBFF" },
      { token: "attribute.name", foreground: "F5DF9A" },
      { token: "attribute.value", foreground: "B8DF8B" },
      { token: "delimiter", foreground: "9FB2BC" },
      { token: "emphasis", fontStyle: "italic" },
      { token: "strong", fontStyle: "bold" },
    ],
    colors: {
      "editor.background": "#12202A",
      "editor.foreground": "#EDF3F5",
      "editor.lineHighlightBackground": "#163244",
      "editor.selectionBackground": "#28506A",
      "editor.inactiveSelectionBackground": "#1B3344",
      "editorCursor.foreground": "#FFFDF7",
      "editorLineNumber.foreground": "#5D7480",
      "editorLineNumber.activeForeground": "#D5E6EE",
      "editorWhitespace.foreground": "#233947",
      "editorIndentGuide.background1": "#233947",
      "editorIndentGuide.activeBackground1": "#456170",
      "editorSuggestWidget.background": "#152431",
      "editorSuggestWidget.border": "#2B4556",
      "editorSuggestWidget.foreground": "#EDF3F5",
      "editorSuggestWidget.selectedBackground": "#223949",
      "editorWidget.background": "#152431",
      "editorWidget.border": "#2B4556",
      "scrollbarSlider.background": "#36505D66",
      "scrollbarSlider.hoverBackground": "#4A697A88",
      "scrollbarSlider.activeBackground": "#5D819599",
    },
  });

  themeRegistered = true;
}

function ensureMarkdownCompletionProvider(): void {
  if (completionRegistered) {
    return;
  }

  monaco.languages.registerCompletionItemProvider("markdown", {
    triggerCharacters: ["#", "-", "[", "`", "!", ">", "|"],
    provideCompletionItems(model, position) {
      const word = model.getWordUntilPosition(position);
      const range = new monaco.Range(
        position.lineNumber,
        word.startColumn,
        position.lineNumber,
        word.endColumn,
      );

      return {
        suggestions: snippetDefinitions.map((snippet, index) => ({
          detail: snippet.description,
          documentation: snippet.description,
          insertText: snippet.insertText,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          kind: monaco.languages.CompletionItemKind.Snippet,
          label: snippet.label,
          range,
          sortText: `0${index}`,
        })),
      };
    },
  });

  completionRegistered = true;
}

export function createMarkdownEditor(container: HTMLElement, initialValue: string): MarkdownEditor {
  ensureTheme();
  ensureMarkdownCompletionProvider();

  const model = monaco.editor.createModel(initialValue, "markdown");
  const editor = monaco.editor.create(container, {
    ariaLabel: "Markdown input",
    automaticLayout: true,
    fontFamily: '"SFMono-Regular", "JetBrains Mono", Menlo, Monaco, Consolas, monospace',
    fontLigatures: false,
    fontSize: 15,
    guides: {
      indentation: false,
    },
    lineDecorationsWidth: 12,
    lineNumbers: "on",
    lineNumbersMinChars: 3,
    minimap: {
      enabled: false,
    },
    model,
    overviewRulerBorder: false,
    padding: {
      bottom: 20,
      top: 20,
    },
    placeholder: "Start writing Markdown...",
    quickSuggestions: {
      comments: true,
      other: true,
      strings: true,
    },
    renderLineHighlight: "line",
    roundedSelection: true,
    scrollBeyondLastLine: false,
    scrollbar: {
      alwaysConsumeMouseWheel: false,
      horizontalScrollbarSize: 8,
      verticalScrollbarSize: 8,
    },
    suggest: {
      insertMode: "insert",
      preview: true,
      selectionMode: "whenQuickSuggestion",
      showIcons: false,
      snippetsPreventQuickSuggestions: false,
    },
    suggestOnTriggerCharacters: true,
    tabSize: 2,
    theme: "ox-playground",
    wordBasedSuggestions: "currentDocument",
    wordWrap: "on",
  });

  return {
    focus(): void {
      editor.focus();
    },
    getValue(): string {
      return editor.getValue();
    },
    onDidChangeValue(listener: () => void): monaco.IDisposable {
      return editor.onDidChangeModelContent(() => {
        listener();
      });
    },
    setValue(value: string): void {
      if (value === editor.getValue()) {
        return;
      }

      editor.setValue(value);
    },
  };
}
