import type { Element } from "hast";
import { formatLineRange } from "./source";
import type { GitHubLineRange, GitHubOptions, GitHubSourceData } from "./types";

function normalizeSourceLines(content: string): string[] {
  const lines = content.replace(/\r\n?/g, "\n").split("\n");
  if (lines.length > 1 && lines.at(-1) === "") {
    lines.pop();
  }
  return lines.length > 0 ? lines : [""];
}

export function createGitHubSourceCard(
  source: GitHubSourceData,
  lines: GitHubLineRange | undefined,
  options: Required<GitHubOptions>,
): Element {
  const allLines = normalizeSourceLines(source.content);
  const start = Math.min(lines?.start ?? 1, allLines.length);
  const end = lines
    ? Math.min(lines.end, allLines.length)
    : Math.min(allLines.length, options.maxSourceLines);
  const selectedLines = allLines.slice(start - 1, end);
  const lineRange = { start, end };
  const loc = selectedLines.length;
  const rangeLabel = formatLineRange(lineRange);
  const locLabel =
    !lines && end < allLines.length
      ? `${rangeLabel} of ${allLines.length} LOC`
      : `${rangeLabel} - ${loc} LOC`;
  const languageClass = source.language ? [`language-${source.language}`] : [];

  return {
    type: "element",
    tagName: "figure",
    properties: {
      className: ["ox-github-code"],
      "data-loc": String(loc),
      "data-source": source.permalink,
    },
    children: [
      {
        type: "element",
        tagName: "figcaption",
        properties: { className: ["ox-github-code-header"] },
        children: [
          {
            type: "element",
            tagName: "a",
            properties: {
              className: ["ox-github-code-title"],
              href: source.permalink,
              target: "_blank",
              rel: "noopener noreferrer",
            },
            children: [{ type: "text", value: `${source.repo}/${source.path}` }],
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-code-loc"] },
            children: [{ type: "text", value: locLabel }],
          },
        ],
      },
      {
        type: "element",
        tagName: "pre",
        properties: {
          className: ["ox-github-code-block", ...languageClass],
          ...(source.language ? { "data-language": source.language } : {}),
        },
        children: [
          {
            type: "element",
            tagName: "code",
            properties: { className: languageClass },
            children: selectedLines.map((line, index) => {
              const lineNumber = start + index;
              return {
                type: "element" as const,
                tagName: "span",
                properties: {
                  className: ["line", "ox-github-code-line"],
                  "data-line": String(lineNumber),
                },
                children: [
                  {
                    type: "element" as const,
                    tagName: "span",
                    properties: { className: ["ox-github-code-line-number"] },
                    children: [{ type: "text" as const, value: String(lineNumber) }],
                  },
                  {
                    type: "element" as const,
                    tagName: "span",
                    properties: { className: ["ox-github-code-line-content"] },
                    children: [{ type: "text" as const, value: line || " " }],
                  },
                ],
              };
            }),
          },
        ],
      },
    ],
  };
}
