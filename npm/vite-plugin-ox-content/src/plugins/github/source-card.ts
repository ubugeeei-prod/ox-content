import type { Element, Text } from "hast";
import { formatLineRange } from "./source";
import type { GitHubLineRange, GitHubOptions, GitHubSourceCommit, GitHubSourceData } from "./types";

function normalizeSourceLines(content: string): string[] {
  const lines = content.replace(/\r\n?/g, "\n").split("\n");
  if (lines.length > 1 && lines.at(-1) === "") {
    lines.pop();
  }
  return lines.length > 0 ? lines : [""];
}

function text(value: string): Text {
  return { type: "text", value };
}

function createSourceLines(lines: string[], start: number): Array<Element | Text> {
  return lines.flatMap((line, index) => {
    const lineNumber = start + index;
    const span: Element = {
      type: "element",
      tagName: "span",
      properties: {
        className: ["line"],
        "data-line": String(lineNumber),
        "data-line-number": String(lineNumber),
      },
      children: [text(line)],
    };
    return index === 0 ? [span] : [text("\n"), span];
  });
}

function createCommitMeta(commit: GitHubSourceCommit): Element {
  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-github-code-commit"],
      href: commit.html_url,
      target: "_blank",
      rel: "noopener noreferrer",
      title: commit.message,
    },
    children: [
      {
        type: "element",
        tagName: "span",
        properties: { className: ["ox-github-code-sha"] },
        children: [text(commit.sha.slice(0, 7))],
      },
      {
        type: "element",
        tagName: "span",
        properties: { className: ["ox-github-code-commit-message"] },
        children: [text(commit.message)],
      },
    ],
  };
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
  const loc = selectedLines.length;
  const rangeLabel = formatLineRange({ start, end });
  const locLabel =
    !lines && end < allLines.length
      ? `${rangeLabel} of ${allLines.length} LOC`
      : `${rangeLabel} · ${loc} LOC`;
  const languageClass = source.language ? [`language-${source.language}`] : [];
  const heading: Element[] = [
    {
      type: "element",
      tagName: "a",
      properties: {
        className: ["ox-github-code-title"],
        href: source.permalink,
        target: "_blank",
        rel: "noopener noreferrer",
      },
      children: [text(`${source.repo}/${source.path}`)],
    },
  ];
  if (source.commit) {
    heading.push(createCommitMeta(source.commit));
  }

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
            tagName: "div",
            properties: { className: ["ox-github-code-heading"] },
            children: heading,
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-code-loc"] },
            children: [text(locLabel)],
          },
        ],
      },
      {
        type: "element",
        tagName: "pre",
        properties: {
          className: [
            "ox-github-code-block",
            "ox-code-block",
            "line-numbers-mode",
            ...languageClass,
          ],
          "data-line-numbers": "true",
          "data-line-number-start": String(start),
          ...(source.language ? { "data-language": source.language } : {}),
        },
        children: [
          {
            type: "element",
            tagName: "code",
            properties: { className: languageClass },
            children: createSourceLines(selectedLines, start),
          },
        ],
      },
    ],
  };
}
