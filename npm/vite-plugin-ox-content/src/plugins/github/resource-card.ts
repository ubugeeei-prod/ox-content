import type { Element, Text } from "hast";
import type { GitHubResourceData, GitHubResourceRef } from "./types";
import { resourceKindLabel } from "./resource";

function text(value: string): Text {
  return { type: "text", value };
}

function iconPath(d: string): Element {
  return {
    type: "element",
    tagName: "svg",
    properties: { viewBox: "0 0 16 16", fill: "currentColor" },
    children: [{ type: "element", tagName: "path", properties: { d }, children: [] }],
  };
}

function fallbackTitle(resource: GitHubResourceRef): string {
  if (resource.kind === "gist") {
    return `Gist ${resource.gistId}`;
  }
  if (resource.kind === "commit") {
    return `${resource.repo}@${resource.sha?.slice(0, 7)}`;
  }
  return `${resource.repo}#${resource.number}`;
}

function fallbackData(resource: GitHubResourceRef): GitHubResourceData {
  return {
    kind: resource.kind,
    permalink: resource.permalink,
    html_url: resource.permalink,
    title: fallbackTitle(resource),
    repo: resource.repo,
    number: resource.number,
    sha: resource.sha,
    gistId: resource.gistId,
    author: resource.gistOwner,
  };
}

export function createGitHubResourceFallbackCard(resource: GitHubResourceRef): Element {
  return createGitHubResourceCard(fallbackData(resource), true);
}

export function createGitHubResourceCard(
  resource: GitHubResourceData,
  unavailable = false,
): Element {
  const kindLabel = resourceKindLabel(resource.kind);
  const className = [
    "ox-github-card",
    "ox-github-resource-card",
    `ox-github-resource-card--${resource.kind}`,
    ...(unavailable ? ["error"] : []),
  ];
  const repoLabel = resource.repo ?? (resource.author ? `${resource.author}/gist` : "gist");
  const ariaTarget = resource.repo ? `${repoLabel} ${kindLabel}` : kindLabel;

  return {
    type: "element",
    tagName: "a",
    properties: {
      className,
      href: resource.html_url,
      target: "_blank",
      rel: "noopener noreferrer",
      "aria-label": `GitHub ${ariaTarget}: ${resource.title}`,
    },
    children: [
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-github-header"] },
        children: [
          {
            ...iconPath(
              "M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z",
            ),
            properties: {
              className: ["ox-github-icon"],
              viewBox: "0 0 16 16",
              fill: "currentColor",
            },
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-repo"] },
            children: [text(repoLabel)],
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-resource-kind"] },
            children: [text(resourceLabel(resource, kindLabel))],
          },
        ],
      },
      {
        type: "element",
        tagName: "p",
        properties: { className: ["ox-github-resource-title"] },
        children: [text(resource.title)],
      },
      ...(resource.body
        ? [
            {
              type: "element" as const,
              tagName: "p",
              properties: { className: ["ox-github-description"] },
              children: [text(resource.body)],
            },
          ]
        : []),
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-github-stats"] },
        children: createMetaChildren(resource, unavailable),
      },
    ],
  };
}

function resourceLabel(resource: GitHubResourceData, kindLabel: string): string {
  if (resource.number) return `${kindLabel} #${resource.number}`;
  if (resource.sha) return `${kindLabel} ${resource.sha.slice(0, 7)}`;
  return kindLabel;
}

function createMetaChildren(
  resource: GitHubResourceData,
  unavailable: boolean,
): Element["children"] {
  const children: Element["children"] = [];
  if (unavailable) {
    children.push(meta("Unavailable"));
  }
  if (resource.state) {
    children.push(meta(resource.state));
  }
  if (resource.author) {
    children.push(meta(`@${resource.author}`));
  }
  if (resource.dateLabel) {
    children.push(meta(resource.dateLabel));
  }
  if (resource.labels?.length) {
    children.push(meta(resource.labels.join(", ")));
  }
  if (resource.files?.length) {
    children.push(meta(resource.files.join(", ")));
  }
  if (typeof resource.comments === "number") {
    children.push(meta(`${resource.comments} comments`));
  }
  return children.length ? children : [meta("Open on GitHub")];
}

function meta(value: string): Element {
  return {
    type: "element",
    tagName: "span",
    properties: { className: ["ox-github-stat"] },
    children: [text(value)],
  };
}
