import type { Element } from "hast";
import type { OgpData } from "./types";
import { extractDomain, isSafeOgpUrl } from "./url";

export function createOgpCard(data: OgpData): Element {
  const children: Element["children"] = [];
  const contentChildren: Element["children"] = [];

  contentChildren.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-title"] },
    children: [{ type: "text", value: data.title }],
  });

  if (data.description) {
    contentChildren.push({
      type: "element",
      tagName: "div",
      properties: { className: ["ox-ogp-description"] },
      children: [{ type: "text", value: data.description }],
    });
  }

  const metaChildren: Element["children"] = [];

  if (data.favicon) {
    metaChildren.push({
      type: "element",
      tagName: "img",
      properties: {
        className: ["ox-ogp-favicon"],
        src: data.favicon,
        alt: "",
        loading: "lazy",
      },
      children: [],
    });
  }

  metaChildren.push({
    type: "element",
    tagName: "span",
    properties: { className: ["ox-ogp-domain"] },
    children: [{ type: "text", value: data.siteName || extractDomain(data.url) }],
  });

  contentChildren.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-meta"] },
    children: metaChildren,
  });

  children.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-content"] },
    children: contentChildren,
  });

  if (data.image) {
    children.push({
      type: "element",
      tagName: "img",
      properties: {
        className: ["ox-ogp-image"],
        src: data.image,
        alt: "",
        loading: "lazy",
      },
      children: [],
    });
  }

  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-ogp-card"],
      href: isSafeOgpUrl(data.url) ? data.url : "#",
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children,
  };
}

export function createFallbackCard(url: string): Element {
  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-ogp-simple"],
      href: isSafeOgpUrl(url) ? url : "#",
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children: [
      {
        type: "element",
        tagName: "svg",
        properties: {
          viewBox: "0 0 24 24",
          fill: "none",
          stroke: "currentColor",
          "stroke-width": "2",
        },
        children: [
          {
            type: "element",
            tagName: "path",
            properties: {
              d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3",
            },
            children: [],
          },
        ],
      },
      { type: "text", value: extractDomain(url) },
    ],
  };
}
