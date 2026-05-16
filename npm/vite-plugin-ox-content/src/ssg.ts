/**
 * SSG (Static Site Generation) module for ox-content
 */

import { createHash } from "node:crypto";
import * as fs from "fs/promises";
import * as path from "path";
import { glob } from "glob";
import { transformMarkdown } from "./transform";
import { generateOgImages } from "./og-image";
import type { OgImagePageEntry } from "./og-image";
import { transformAllPlugins } from "./plugins";
import type { TransformAllOptions } from "./plugins";
import { protectMermaidSvgs, restoreMermaidSvgs } from "./plugins/mermaid-protect";
import { transformIslands, hasIslands } from "./island";
import { importNapiModule } from "./napi";
import type {
  ResolvedOptions,
  ResolvedSsgOptions,
  SsgOptions,
  TocEntry,
  HeroConfig,
  FeatureConfig,
} from "./types";
import { resolveTheme, themeToNapi } from "./theme";
import type { ResolvedThemeConfig, SidebarItem } from "./theme";

/**
 * Navigation item for SSG.
 */
export interface SsgNavItem {
  title: string;
  path: string;
  href: string;
  children?: SsgNavItem[];
  collapsed?: boolean;
}

/**
 * Entry page configuration for SSG (passed to Rust).
 */
export interface SsgEntryPageConfig {
  hero?: HeroConfig;
  features?: FeatureConfig[];
}

/**
 * Page data for SSG.
 */
export interface SsgPageData {
  title: string;
  description?: string;
  content: string;
  toc: TocEntry[];
  lastUpdated?: number;
  frontmatter: Record<string, unknown>;
  path: string;
  href: string;
  /** Entry page configuration (if layout: entry) */
  entryPage?: SsgEntryPageConfig;
}

/**
 * Default HTML template for SSG pages with navigation.
 */
export const DEFAULT_HTML_TEMPLATE = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{title}}{{#siteName}} - {{siteName}}{{/siteName}}</title>
  {{#description}}<meta name="description" content="{{description}}">{{/description}}
  <!-- Open Graph -->
  <meta property="og:type" content="website">
  <meta property="og:title" content="{{title}}{{#siteName}} - {{siteName}}{{/siteName}}">
  {{#description}}<meta property="og:description" content="{{description}}">{{/description}}
  {{#ogImage}}<meta property="og:image" content="{{ogImage}}">{{/ogImage}}
  <!-- Twitter Card -->
  {{#ogImage}}<meta name="twitter:card" content="summary_large_image">{{/ogImage}}
  {{^ogImage}}<meta name="twitter:card" content="summary">{{/ogImage}}
  <meta name="twitter:title" content="{{title}}{{#siteName}} - {{siteName}}{{/siteName}}">
  {{#description}}<meta name="twitter:description" content="{{description}}">{{/description}}
  {{#ogImage}}<meta name="twitter:image" content="{{ogImage}}">{{/ogImage}}
  <style>
    :root {
      --sidebar-width: 260px;
      --header-height: 60px;
      --max-content-width: 960px;
      --font-sans: 'IBM Plex Sans', 'Avenir Next', 'Segoe UI Variable', 'Segoe UI', sans-serif;
      --font-mono: 'IBM Plex Mono', 'SFMono-Regular', Consolas, monospace;
      --color-bg: #ffffff;
      --color-bg-alt: #f5f7fb;
      --color-text: #131a30;
      --color-text-muted: #4f607b;
      --color-border: #d2dbea;
      --color-primary: #4f6fae;
      --color-primary-hover: #425f96;
      --color-code-bg: #101a31;
      --color-code-bg-top: #18264a;
      --color-code-text: #edf3ff;
      --color-code-line-highlight: rgba(56, 189, 248, 0.16);
      --color-code-line-warning: rgba(245, 158, 11, 0.18);
      --color-code-line-warning-border: #f59e0b;
      --color-code-line-error: rgba(239, 68, 68, 0.18);
      --color-code-line-error-border: #ef4444;
      --color-code-frame-border: rgba(147, 166, 200, 0.46);
      --surface-noise-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 180 180'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.2' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='180' height='180' filter='url(%23noise)' opacity='0.062'/%3E%3C/svg%3E");
      --surface-noise-size: 164px 164px;
    }
    [data-theme="dark"] {
      --color-bg: #060816;
      --color-bg-alt: #0d1528;
      --color-text: #ebf2ff;
      --color-text-muted: #8ea0bf;
      --color-border: #223252;
      --color-primary: #86a4da;
      --color-primary-hover: #a3bbe8;
      --color-code-bg: #0a1020;
      --color-code-bg-top: #0a1020;
      --color-code-text: #e7f0ff;
      --color-code-line-highlight: rgba(14, 165, 233, 0.2);
      --color-code-line-warning: rgba(245, 158, 11, 0.2);
      --color-code-line-warning-border: #f59e0b;
      --color-code-line-error: rgba(239, 68, 68, 0.22);
      --color-code-line-error-border: #f87171;
      --color-code-frame-border: rgba(34, 50, 82, 0.92);
      --surface-noise-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 180 180'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.25' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='180' height='180' filter='url(%23noise)' opacity='0.098'/%3E%3C/svg%3E");
    }
    @media (prefers-color-scheme: dark) {
      :root:not([data-theme="light"]) {
        --color-bg: #060816;
        --color-bg-alt: #0d1528;
        --color-text: #ebf2ff;
        --color-text-muted: #8ea0bf;
        --color-border: #223252;
        --color-primary: #86a4da;
        --color-primary-hover: #a3bbe8;
        --color-code-bg: #0a1020;
        --color-code-bg-top: #0a1020;
        --color-code-text: #e7f0ff;
        --color-code-line-highlight: rgba(14, 165, 233, 0.2);
        --color-code-line-warning: rgba(245, 158, 11, 0.2);
        --color-code-line-warning-border: #f59e0b;
        --color-code-line-error: rgba(239, 68, 68, 0.22);
        --color-code-line-error-border: #f87171;
        --color-code-frame-border: rgba(34, 50, 82, 0.92);
        --surface-noise-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 180 180'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.25' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='180' height='180' filter='url(%23noise)' opacity='0.098'/%3E%3C/svg%3E");
      }
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    html { scroll-behavior: smooth; }
    body {
      font-family: var(--font-sans);
      line-height: 1.7;
      color: var(--color-text);
      background: var(--color-bg);
      background-image: var(--surface-noise-image);
      background-size: var(--surface-noise-size);
      background-repeat: repeat;
      background-blend-mode: soft-light;
    }
    a { color: var(--color-primary); text-decoration: none; }
    a:hover { color: var(--color-primary-hover); text-decoration: underline; }

    /* Header */
    .header {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      height: var(--header-height);
      background: var(--color-bg);
      border-bottom: 1px solid var(--color-border);
      display: flex;
      align-items: center;
      padding: 0 1.5rem;
      z-index: 100;
    }
    .header,
    .sidebar,
    .search-modal,
    .mobile-footer,
    .content .ox-api-entry,
    .content .ox-api-module,
    .content blockquote.ox-callout {
      background-image: var(--surface-noise-image);
      background-size: var(--surface-noise-size);
      background-repeat: repeat;
      background-blend-mode: soft-light;
    }
    .header-title {
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--color-text);
    }
    .header-title:hover { text-decoration: none; }
    .menu-toggle {
      display: none;
      background: none;
      border: none;
      cursor: pointer;
      padding: 0.5rem;
      margin-right: 0.75rem;
    }
    .menu-toggle svg { display: block; }
    .menu-toggle path { stroke: var(--color-text); }
    .header-actions { margin-left: auto; display: flex; align-items: center; gap: 0.5rem; }
    .search-button {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 0.75rem;
      background: var(--color-bg-alt);
      border: 1px solid var(--color-border);
      border-radius: 4px;
      color: var(--color-text-muted);
      cursor: pointer;
      font-size: 0.875rem;
      transition: border-color 0.15s, color 0.15s;
    }
    .search-button:hover { border-color: var(--color-primary); color: var(--color-text); }
    .search-button svg { width: 16px; height: 16px; }
    .search-button kbd {
      padding: 0.125rem 0.375rem;
      background: var(--color-bg);
      border: 1px solid var(--color-border);
      border-radius: 4px;
      font-family: var(--font-mono);
      font-size: 0.75rem;
    }
    @media (max-width: 640px) {
      .search-button span, .search-button kbd { display: none; }
      .search-button { padding: 0.5rem; }
    }
    .search-modal-overlay {
      display: none;
      position: fixed;
      inset: 0;
      z-index: 200;
      background: rgba(0,0,0,0.6);
      justify-content: center;
      padding-top: 10vh;
    }
    .search-modal-overlay.open { display: flex; }
    .search-modal {
      width: 100%;
      max-width: 560px;
      margin: 0 1rem;
      background: var(--color-bg);
      border: 1px solid var(--color-border);
      border-radius: 4px;
      overflow: hidden;
      max-height: 70vh;
      display: flex;
      flex-direction: column;
    }
    .search-header {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1rem;
      border-bottom: 1px solid var(--color-border);
    }
    .search-header svg { flex-shrink: 0; color: var(--color-text-muted); }
    .search-input {
      flex: 1;
      background: none;
      border: none;
      outline: none;
      font-size: 1rem;
      color: var(--color-text);
    }
    .search-input::placeholder { color: var(--color-text-muted); }
    .search-close {
      padding: 0.25rem 0.5rem;
      background: var(--color-bg-alt);
      border: 1px solid var(--color-border);
      border-radius: 4px;
      color: var(--color-text-muted);
      font-family: var(--font-mono);
      font-size: 0.75rem;
      cursor: pointer;
    }
    .search-results {
      flex: 1;
      overflow-y: auto;
      padding: 0.5rem;
    }
    .search-result {
      display: block;
      padding: 0.75rem 1rem;
      border-radius: 4px;
      color: var(--color-text);
      text-decoration: none;
    }
    .search-result:hover, .search-result.selected { background: var(--color-bg-alt); text-decoration: none; }
    .search-result-title { font-weight: 600; font-size: 0.875rem; margin-bottom: 0.25rem; }
    .search-result-snippet { font-size: 0.8125rem; color: var(--color-text-muted); }
    .search-empty { padding: 2rem 1rem; text-align: center; color: var(--color-text-muted); }
    .search-footer {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 1rem;
      padding: 0.75rem 1rem;
      border-top: 1px solid var(--color-border);
      background: var(--color-bg-alt);
      font-size: 0.75rem;
      color: var(--color-text-muted);
    }
    .search-footer kbd {
      padding: 0.125rem 0.375rem;
      background: var(--color-bg);
      border: 1px solid var(--color-border);
      border-radius: 4px;
      font-family: var(--font-mono);
    }
    .theme-toggle {
      background: none;
      border: none;
      cursor: pointer;
      padding: 0.5rem;
      border-radius: 4px;
      color: var(--color-text-muted);
      transition: background 0.15s, color 0.15s;
    }
    .theme-toggle:hover { background: var(--color-bg-alt); color: var(--color-text); }
    .theme-toggle svg { display: block; width: 20px; height: 20px; }
    .theme-toggle .icon-sun { display: none; }
    .theme-toggle .icon-moon { display: block; }
    [data-theme="dark"] .theme-toggle .icon-sun { display: block; }
    [data-theme="dark"] .theme-toggle .icon-moon { display: none; }
    @media (prefers-color-scheme: dark) {
      :root:not([data-theme="light"]) .theme-toggle .icon-sun { display: block; }
      :root:not([data-theme="light"]) .theme-toggle .icon-moon { display: none; }
    }

    /* Layout */
    .layout {
      display: flex;
      padding-top: var(--header-height);
      min-height: 100vh;
    }

    /* Sidebar */
    .sidebar {
      position: fixed;
      top: var(--header-height);
      left: 0;
      bottom: 0;
      width: var(--sidebar-width);
      background: color-mix(in srgb, var(--color-bg-alt) 16%, var(--color-bg));
      border-right: 1px solid color-mix(in srgb, var(--color-border) 48%, transparent);
      overflow-y: auto;
      padding: 1rem 0.875rem 1.5rem;
    }
    .sidebar--entry { display: none; }
    .sidebar nav {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    .nav-section { margin-bottom: 0; }
    .nav-title {
      font-size: 0.6875rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--color-text-muted);
      margin-bottom: 0.4rem;
      padding: 0 0.625rem;
    }
    .nav-list {
      list-style: none;
      display: flex;
      flex-direction: column;
      gap: 0.125rem;
    }
    .nav-item { margin: 0; }
    .nav-link {
      display: block;
      padding: 0.45rem 0.625rem;
      border-radius: 0;
      color: color-mix(in srgb, var(--color-text) 92%, var(--color-text-muted));
      font-size: 0.875rem;
    }
    .nav-link:hover {
      background: color-mix(in srgb, var(--color-bg-alt) 58%, transparent);
      color: var(--color-text);
      text-decoration: none;
    }
    .nav-link.active {
      background: color-mix(in srgb, var(--color-bg-alt) 72%, transparent);
      color: var(--color-text);
      font-weight: 600;
    }

    /* Main content */
    .main {
      flex: 1;
      margin-left: var(--sidebar-width);
      padding: 2rem;
      min-width: 0;
      overflow-x: hidden;
    }
    .content {
      max-width: var(--max-content-width);
      margin: 0 auto;
      overflow-wrap: break-word;
      word-wrap: break-word;
      word-break: break-word;
    }

    /* TOC (right sidebar) */
    .toc {
      position: fixed;
      top: calc(var(--header-height) + 2rem);
      right: 2rem;
      width: 200px;
      font-size: 0.8125rem;
    }
    .toc-title {
      font-weight: 600;
      margin-bottom: 0.75rem;
      color: var(--color-text-muted);
    }
    .toc-list { list-style: none; }
    .toc-item { margin: 0.375rem 0; }
    .toc-link {
      color: var(--color-text-muted);
      display: block;
      padding-left: calc((var(--depth, 1) - 1) * 0.75rem);
    }
    .toc-link:hover { color: var(--color-primary); }
    @media (max-width: 1200px) { .toc { display: none; } }

    /* Typography */
    .content h1 { font-size: 2.25rem; margin-bottom: 1rem; line-height: 1.2; }
    .content h2 { font-size: 1.5rem; margin-top: 2.5rem; margin-bottom: 1rem; padding-bottom: 0.5rem; border-bottom: 1px solid var(--color-border); }
    .content h3 { font-size: 1.25rem; margin-top: 2rem; margin-bottom: 0.75rem; }
    .content h4 { font-size: 1rem; margin-top: 1.5rem; margin-bottom: 0.5rem; }
    .content p { margin-bottom: 1rem; }
    .content ul, .content ol { margin: 1rem 0; padding-left: 1.5rem; }
    .content li { margin: 0.375rem 0; }
    .content blockquote {
      border-left: 4px solid var(--color-primary);
      padding: 0.5rem 1rem;
      margin: 1rem 0;
      background: var(--color-bg-alt);
      border-radius: 0 4px 4px 0;
    }
    .content blockquote.ox-callout {
      --callout-accent: var(--color-primary);
      border-left-width: 3px;
      border-left-color: var(--callout-accent);
      padding: 0.9rem 1rem;
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 92%, var(--callout-accent) 8%);
    }
    .content blockquote.ox-callout.ox-callout--note,
    .content blockquote.ox-callout.ox-callout--important { --callout-accent: var(--color-primary); }
    .content blockquote.ox-callout.ox-callout--tip { --callout-accent: #0891b2; }
    .content blockquote.ox-callout.ox-callout--warning { --callout-accent: #d97706; }
    .content blockquote.ox-callout.ox-callout--caution { --callout-accent: #dc2626; }
    .content .ox-callout-title {
      margin: 0 0 0.5rem;
      font-size: 0.75rem;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--callout-accent, var(--color-primary));
    }
    .content blockquote.ox-callout > :last-child { margin-bottom: 0; }
    .content blockquote.ox-callout > :not(.ox-callout-title):first-of-type { margin-top: 0; }
    .content code {
      font-family: var(--font-mono);
      font-size: 0.875em;
      background: var(--color-bg-alt);
      padding: 0.2em 0.4em;
      border-radius: 4px;
      word-break: break-all;
    }
    .content pre {
      background: linear-gradient(
        180deg,
        var(--color-code-bg-top) 0,
        var(--color-code-bg) 3.5rem
      ) !important;
      color: var(--color-code-text);
      padding: 1rem 1.25rem;
      border-radius: 4px;
      border: 1px solid var(--color-code-frame-border);
      overflow-x: auto;
      margin: 1.5rem 0;
      line-height: 1.5;
    }
    .content pre code {
      background: transparent;
      border: 0;
      padding: 0;
      border-radius: 0;
      font-size: 0.8125rem;
      word-break: normal;
    }
    .content pre.ox-code-block code {
      display: block;
    }
    .content pre.ox-code-block .line {
      display: block;
      margin: 0 -1.25rem;
      padding: 0 1.25rem;
    }
    .content pre.ox-code-block .ox-code-line--highlight {
      background: var(--color-code-line-highlight);
    }
    .content pre.ox-code-block .ox-code-line--warning {
      background: var(--color-code-line-warning);
      border-left: 3px solid var(--color-code-line-warning-border);
    }
    .content pre.ox-code-block .ox-code-line--error {
      background: var(--color-code-line-error);
      border-left: 3px solid var(--color-code-line-error-border);
    }
    .content table {
      width: 100%;
      border-collapse: collapse;
      margin: 1.5rem 0;
      font-size: 0.875rem;
    }
    .content th, .content td {
      border: 1px solid var(--color-border);
      padding: 0.75rem 1rem;
      text-align: left;
    }
    .content th { background: var(--color-bg-alt); font-weight: 600; }
    .content img { max-width: 100%; height: auto; border-radius: 4px; display: block; }
    .content img[alt*="Logo"] { max-width: 200px; display: block; margin: 1rem 0; }
    .content img[alt*="Architecture"] { max-width: 600px; }
    .content img[alt*="Benchmark"] { max-width: 680px; }
    .content hr { border: none; border-top: 1px solid var(--color-border); margin: 2rem 0; }
    .content .ox-api-controls {
      display: flex;
      justify-content: flex-end;
      align-items: center;
      gap: 0.5rem;
      margin: 0 0 1rem;
    }
    .content .ox-api-controls__button {
      appearance: none;
      border: 1px solid color-mix(in srgb, var(--color-border) 82%, transparent);
      background: color-mix(in srgb, var(--color-bg-alt) 82%, var(--color-bg));
      padding: 0.4rem 0.7rem;
      border-radius: 4px;
      color: color-mix(in srgb, var(--color-text) 82%, var(--color-text-muted));
      font-family: var(--font-mono);
      font-size: 0.78rem;
      font-weight: 600;
      line-height: 1.4;
      cursor: pointer;
    }
    .content .ox-api-controls__button:hover {
      color: var(--color-primary);
      border-color: color-mix(in srgb, var(--color-primary) 38%, var(--color-border));
      background: color-mix(in srgb, var(--color-bg-alt) 68%, var(--color-primary) 6%);
    }
    .content .ox-api-stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(7rem, 1fr));
      gap: 0.55rem;
      margin: 1rem 0 1.35rem;
    }
    .content .ox-api-stat {
      min-width: 0;
      padding: 0.7rem 0.75rem;
      border: 1px solid color-mix(in srgb, var(--color-border) 76%, transparent);
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 72%, transparent);
    }
    .content .ox-api-stat strong {
      display: block;
      font-family: var(--font-mono);
      font-size: 1rem;
      line-height: 1.2;
      color: var(--color-text);
    }
    .content .ox-api-stat span {
      display: block;
      margin-top: 0.25rem;
      font-family: var(--font-mono);
      font-size: 0.7rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--color-text-muted);
    }
    .content .ox-api-stat--warning {
      border-color: color-mix(in srgb, #f59e0b 42%, var(--color-border));
      background: color-mix(in srgb, var(--color-bg-alt) 78%, #f59e0b 8%);
    }
    .content .ox-api-entry,
    .content .ox-api-module {
      margin: 0;
      border: 0;
      border-top: 1px solid color-mix(in srgb, var(--color-border) 74%, transparent);
      border-radius: 0;
      background: transparent;
      overflow: visible;
    }
    .content .ox-api-entry:last-child,
    .content .ox-api-module:last-child {
      border-bottom: 1px solid color-mix(in srgb, var(--color-border) 74%, transparent);
    }
    .content .ox-api-entry summary,
    .content .ox-api-module summary {
      list-style: none;
      cursor: pointer;
      padding: 1rem 0;
      position: relative;
    }
    .content .ox-api-entry summary::-webkit-details-marker,
    .content .ox-api-module summary::-webkit-details-marker { display: none; }
    .content .ox-api-entry summary {
      display: grid;
      grid-template-columns: var(--octc-api-kind-width, 6.5rem) minmax(0, 1fr) auto;
      align-items: start;
      gap: 0.95rem;
    }
    .content .ox-api-entry summary::after,
    .content .ox-api-module summary::after {
      content: "+";
      align-self: center;
      color: var(--color-text-muted);
      font-family: var(--font-mono);
      font-size: 0.95rem;
      font-weight: 600;
      line-height: 1;
    }
    .content .ox-api-entry[open] summary::after,
    .content .ox-api-module[open] summary::after {
      content: "−";
      color: var(--color-primary);
    }
    .content .ox-api-entry[open] summary,
    .content .ox-api-module[open] summary {
      border-bottom: 1px solid color-mix(in srgb, var(--color-border) 72%, transparent);
    }
    .content .ox-api-entry__kind,
    .content .ox-api-module__kind {
      display: block;
      width: var(--octc-api-kind-width, 6.5rem);
      padding: 0.3rem 0 0;
      background: transparent;
      border: 0;
      font-family: var(--font-mono);
      font-size: 0.76rem;
      font-weight: 600;
      letter-spacing: 0.01em;
      text-align: left;
      white-space: nowrap;
      color: var(--color-text-muted);
    }
    .content .ox-api-module__count {
      display: inline-flex;
      align-items: center;
      padding: 0.2rem 0.48rem;
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 84%, var(--color-primary) 8%);
      border: 1px solid color-mix(in srgb, var(--color-border) 82%, transparent);
      color: var(--color-text-muted);
      font-family: var(--font-mono);
      font-size: 0.72rem;
      font-weight: 600;
      letter-spacing: 0.03em;
      white-space: nowrap;
    }
    .content .ox-api-entry__name {
      display: block;
      font-family: var(--font-mono);
      font-size: 0.95rem;
      font-weight: 600;
      line-height: 1.55;
    }
    .content .ox-api-entry__signature,
    .content .ox-api-module__signature {
      display: block;
      width: 100%;
      min-width: 0;
      font-family: var(--font-mono);
      font-size: 0.95rem;
      line-height: 1.55;
      white-space: nowrap;
      overflow-x: auto;
      overflow-y: hidden;
      -webkit-overflow-scrolling: touch;
    }
    .content .ox-api-entry__description {
      display: block;
      color: color-mix(in srgb, var(--color-text) 78%, var(--color-text-muted));
      font-size: 0.9rem;
      line-height: 1.6;
      max-width: 72ch;
    }
    .content .ox-api-entry__summary-main {
      min-width: 0;
      display: flex;
      flex-direction: column;
      gap: 0.55rem;
    }
    .content .ox-api-entry__meta,
    .content .ox-api-module__meta {
      display: flex;
      flex-wrap: wrap;
      gap: 0.35rem;
    }
    .content .ox-api-badge {
      display: inline-flex;
      align-items: center;
      max-width: 100%;
      padding: 0.18rem 0.42rem;
      border: 1px solid color-mix(in srgb, var(--color-border) 78%, transparent);
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 78%, var(--color-primary) 6%);
      color: color-mix(in srgb, var(--color-text) 74%, var(--color-text-muted));
      font-family: var(--font-mono);
      font-size: 0.68rem;
      font-weight: 600;
      line-height: 1.35;
    }
    .content .ox-api-badge--warning {
      border-color: color-mix(in srgb, #f59e0b 46%, var(--color-border));
      background: color-mix(in srgb, var(--color-bg-alt) 78%, #f59e0b 9%);
      color: color-mix(in srgb, var(--color-text) 86%, #f59e0b);
    }
    .content .ox-api-entry__name,
    .content .ox-api-entry__signature,
    .content .ox-api-module__name,
    .content .ox-api-module__signature {
      background: transparent;
      padding: 0;
      border-radius: 0;
      word-break: normal;
    }
    .content code.shiki-inline.ox-api-entry__signature--highlighted,
    .content code.shiki-inline.ox-api-module__signature--highlighted {
      display: block;
      width: 100%;
      max-width: 100%;
      background: linear-gradient(
        180deg,
        var(--color-code-bg-top) 0,
        var(--color-code-bg) 2.75rem
      ) !important;
      border: 1px solid var(--color-code-frame-border) !important;
      padding: 0.55rem 0.7rem !important;
      border-radius: 4px !important;
      white-space: nowrap;
      overflow-x: auto;
      overflow-y: hidden;
      -webkit-overflow-scrolling: touch;
    }
    .content code.shiki-inline.ox-api-entry__signature--highlighted .line,
    .content code.shiki-inline.ox-api-module__signature--highlighted .line {
      display: block;
      width: max-content;
      min-width: 100%;
    }
    .content .ox-api-entry__body,
    .content .ox-api-module__body { padding: 0.7rem 0 1.9rem; }
    .content .ox-api-entry__body {
      margin-left: calc(var(--octc-api-kind-width, 6.5rem) + 0.95rem);
      margin-top: 0.7rem;
      padding: 1.45rem 1rem 2.1rem 1.1rem;
      border: 1px solid color-mix(in srgb, var(--color-border) 72%, transparent);
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 68%, transparent);
    }
    .content .ox-api-entry__body > :first-child { margin-top: 0; }
    .content .ox-api-entry__body > :last-child { margin-bottom: 0; }
    .content .ox-api-entry[open] summary {
      padding-bottom: 0.35rem;
    }
    .content .ox-api-entry__section {
      display: grid;
      grid-template-columns: 6.5rem minmax(0, 1fr);
      gap: 0.2rem 1.25rem;
      align-items: start;
      margin-top: 1rem;
      padding-top: 1rem;
      border-top: 1px solid color-mix(in srgb, var(--color-border) 72%, transparent);
    }
    .content .ox-api-entry__section h4 {
      margin-top: 0;
      margin-bottom: 0;
      padding-top: 0.25rem;
      font-family: var(--font-mono);
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--color-text-muted);
    }
    .content .ox-api-entry__section > :not(h4) { min-width: 0; }
    .content .ox-api-entry__source {
      margin: 0 0 0.15rem;
      font-family: var(--font-mono);
      font-size: 0.78rem;
      color: var(--color-text-muted);
    }
    .content .ox-api-entry__source a {
      color: inherit;
      text-decoration-color: color-mix(in srgb, var(--color-text-muted) 38%, transparent);
    }
    .content .ox-api-entry__tags,
    .content .ox-api-module__list {
      list-style: none;
      padding-left: 0;
      margin: 0;
    }
    .content .ox-api-entry__tags {
      display: flex;
      flex-wrap: wrap;
      gap: 0.6rem;
    }
    .content .ox-api-entry__tags li {
      display: inline-flex;
      align-items: center;
      gap: 0.45rem;
      padding: 0.4rem 0.55rem;
      border: 1px solid color-mix(in srgb, var(--color-border) 80%, transparent);
      border-radius: 4px;
      background: color-mix(in srgb, var(--color-bg-alt) 72%, transparent);
    }
    .content .ox-api-module__list li {
      display: grid;
      grid-template-columns: var(--octc-api-kind-width, 6.5rem) minmax(0, 1fr);
      align-items: start;
      gap: 1rem;
      padding: 0.85rem 0;
      border-top: 1px solid color-mix(in srgb, var(--color-border) 70%, transparent);
    }
    .content .ox-api-module__list li:first-child { border-top: none; }
    .content .ox-api-entry__tag-name,
    .content .ox-api-module__title { font-weight: 700; }
    .content .ox-api-entry__tag-name {
      color: var(--color-primary);
      font-family: var(--font-mono);
      font-size: 0.74rem;
    }
    .content .ox-api-entry__tag-value {
      color: var(--color-text);
      font-size: 0.84rem;
      line-height: 1.45;
    }
    .content .ox-api-module summary {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto auto;
      align-items: center;
      gap: 0.9rem;
    }
    .content .ox-api-module__body { padding-top: 0.15rem; }
    .content .ox-api-module__item { min-width: 0; }
    .content .ox-api-module__link {
      display: block;
      text-decoration: none;
    }
    .content .ox-api-module__link:hover { text-decoration: none; }
    .content .ox-api-module__name,
    .content .ox-api-module__signature {
      display: block;
      font-family: var(--font-mono);
      font-size: 0.91rem;
      line-height: 1.55;
      color: var(--color-text);
    }
    .content .ox-api-module__summary {
      display: block;
      margin-top: 0.45rem;
      color: color-mix(in srgb, var(--color-text) 76%, var(--color-text-muted));
      font-size: 0.88rem;
      line-height: 1.55;
    }
    .content .ox-api-entry__section--signature pre {
      margin: 0;
      border: 1px solid var(--color-code-frame-border);
      border-radius: 4px;
    }
    .content .ox-api-entry__example + .ox-api-entry__example {
      margin-top: 0.85rem;
    }
    .content .ox-api-entry__example-heading {
      margin-bottom: 0.45rem;
      font-family: var(--font-mono);
      font-size: 0.72rem;
      font-weight: 700;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--color-text-muted);
    }
    .content .ox-api-entry__section--examples pre {
      margin: 0;
      border: 1px solid var(--color-code-frame-border);
      border-radius: 4px;
    }
    .content .ox-api-entry__params {
      list-style: none;
      padding: 0;
      margin: 0;
    }
    .content .ox-api-entry__param {
      padding: 0.8rem 0;
      border-top: 1px solid color-mix(in srgb, var(--color-border) 70%, transparent);
    }
    .content .ox-api-entry__param:first-child {
      padding-top: 0;
      border-top: 0;
    }
    .content .ox-api-entry__param-heading {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.5rem;
    }
    .content .ox-api-entry__param-name,
    .content .ox-api-entry__return-type {
      font-family: var(--font-mono);
      font-size: 0.84rem;
      font-weight: 600;
      color: var(--color-text);
      background: color-mix(in srgb, var(--color-bg-alt) 84%, transparent);
      border: 1px solid color-mix(in srgb, var(--color-border) 82%, transparent);
      padding: 0.28rem 0.42rem;
      border-radius: 4px;
    }
    .content .ox-api-entry__param-type {
      font-family: var(--font-mono);
      font-size: 0.78rem;
      color: var(--color-code-text);
      background: color-mix(in srgb, var(--color-code-bg) 96%, #243556);
      border: 1px solid color-mix(in srgb, var(--color-code-bg) 82%, var(--color-border));
      padding: 0.26rem 0.42rem;
      border-radius: 4px;
    }
    .content .ox-api-entry__param-description,
    .content .ox-api-entry__return-description {
      margin: 0.55rem 0 0;
      font-size: 0.88rem;
      line-height: 1.6;
      color: color-mix(in srgb, var(--color-text) 78%, var(--color-text-muted));
    }
    .content .ox-api-entry__return { margin: 0; }

    /* Responsive */
    @media (max-width: 768px) {
      .menu-toggle { display: block; }
      .sidebar {
        transform: translateX(-100%);
        z-index: 99;
        width: 280px;
      }
      .sidebar--entry { display: block; }
      .sidebar.open { transform: translateX(0); }
      .main { margin-left: 0; padding: 1rem 0.75rem; }
      .content { padding: 0 0.25rem; }
      .content h1 { font-size: 1.5rem; line-height: 1.3; margin-bottom: 0.75rem; }
      .content h2 { font-size: 1.2rem; margin-top: 2rem; }
      .content h3 { font-size: 1.1rem; }
      .content p { font-size: 0.9375rem; margin-bottom: 0.875rem; }
      .content ul, .content ol { padding-left: 1.25rem; font-size: 0.9375rem; }
      .content pre {
        padding: 0.75rem;
        font-size: 0.75rem;
        margin: 1rem -0.75rem;
        border-radius: 0;
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
      }
      .content .ox-api-controls {
        justify-content: flex-start;
        gap: 0.75rem;
      }
      .content .ox-api-entry__body {
        margin-left: 0;
        margin-top: 0.55rem;
        padding: 1.2rem 0 1.7rem;
        border-left: 0;
        border-right: 0;
        border-bottom: 0;
        border-radius: 0;
        background: transparent;
      }
      .content .ox-api-entry__section {
        grid-template-columns: 1fr;
        gap: 0.5rem;
      }
      .content .ox-api-entry__tags li,
      .content .ox-api-module__list li,
      .content .ox-api-module summary {
        grid-template-columns: 1fr;
      }
      .content .ox-api-entry__signature { width: 100%; }
      .content pre.ox-code-block .line {
        margin: 0 -0.75rem;
        padding: 0 0.75rem;
      }
      .content code { font-size: 0.8125em; }
      .content table {
        display: block;
        width: max-content;
        min-width: 100%;
        max-width: calc(100vw - 1.5rem);
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
        font-size: 0.8125rem;
        margin: 1rem 0;
        border-collapse: separate;
        border-spacing: 0;
      }
      .content th, .content td { padding: 0.5rem 0.75rem; white-space: nowrap; vertical-align: top; }
      .content img { margin: 1rem 0; }
      .content img[alt*="Logo"] { max-width: 150px; }
      .content img[alt*="Architecture"] { max-width: 100%; }
      .content img[alt*="Benchmark"] { max-width: 100%; }
      .content blockquote { padding: 0.5rem 0.75rem; margin: 1rem 0; font-size: 0.9375rem; }
      .header { padding: 0 1rem; }
      .header-title { font-size: 1rem; }
      .header-title:not(.header-title--logo-only) img { width: 24px; height: 24px; }
      .header-title--logo-only .header-logo { width: 152px; height: auto; }
      .overlay {
        display: none;
        position: fixed;
        inset: 0;
        background: transparent;
        z-index: 98;
      }
      .overlay.open { display: block; }
    }

    /* Extra small devices */
    @media (max-width: 480px) {
      .main { padding: 0.75rem 0.5rem; }
      .content h1 { font-size: 1.35rem; }
      .content pre { font-size: 0.6875rem; padding: 0.625rem; }
      .content table { max-width: calc(100vw - 1rem); font-size: 0.75rem; }
      .content th, .content td { padding: 0.375rem 0.5rem; }
    }
  </style>
</head>
<body>
  <header class="header">
    <button class="menu-toggle" aria-label="Toggle menu">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke-width="2" stroke-linecap="round">
        <path d="M3 12h18M3 6h18M3 18h18"/>
      </svg>
    </button>
    <a href="{{base}}index.html" class="header-title">
      <img src="{{base}}logo.svg" alt="" width="28" height="28" style="margin-right: 8px; vertical-align: middle;" />
      {{siteName}}
    </a>
    <div class="header-actions">
      <button class="search-button" aria-label="Search">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <span>Search</span>
        <kbd>/</kbd>
      </button>
      <button class="theme-toggle" aria-label="Toggle theme">
        <svg class="icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
        </svg>
        <svg class="icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
      </button>
    </div>
  </header>
  <div class="search-modal-overlay">
    <div class="search-modal">
      <div class="search-header">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <input type="text" class="search-input" placeholder="Search documentation..." />
        <button class="search-close">Esc</button>
      </div>
      <div class="search-results"></div>
      <div class="search-footer">
        <span><kbd>↑</kbd><kbd>↓</kbd> to navigate</span>
        <span><kbd>Enter</kbd> to select</span>
        <span><kbd>Esc</kbd> to close</span>
      </div>
    </div>
  </div>
  <div class="overlay"></div>
  <div class="layout">
    <aside class="sidebar{{#entryPage}} sidebar--entry{{/entryPage}}">
      <nav>
{{navigation}}
      </nav>
    </aside>
    <main class="main">
      <article class="content">
{{content}}
      </article>
    </main>
{{#hasToc}}
    <aside class="toc">
      <div class="toc-title">On this page</div>
      <ul class="toc-list">
{{toc}}
      </ul>
    </aside>
{{/hasToc}}
  </div>
  <script>
    // Menu toggle
    const toggle = document.querySelector('.menu-toggle');
    const sidebar = document.querySelector('.sidebar');
    const overlay = document.querySelector('.overlay');
    if (toggle && sidebar && overlay) {
      const close = () => { sidebar.classList.remove('open'); overlay.classList.remove('open'); };
      toggle.addEventListener('click', () => {
        sidebar.classList.toggle('open');
        overlay.classList.toggle('open');
      });
      overlay.addEventListener('click', close);
      sidebar.querySelectorAll('a').forEach(a => a.addEventListener('click', close));
    }

    // Theme toggle
    const themeToggle = document.querySelector('.theme-toggle');
    const getPreferredTheme = () => {
      const stored = localStorage.getItem('theme');
      if (stored) return stored;
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    };
    const setTheme = (theme) => {
      document.documentElement.setAttribute('data-theme', theme);
      localStorage.setItem('theme', theme);
    };
    // Initialize theme
    setTheme(getPreferredTheme());
    if (themeToggle) {
      themeToggle.addEventListener('click', () => {
        const current = document.documentElement.getAttribute('data-theme') || getPreferredTheme();
        setTheme(current === 'dark' ? 'light' : 'dark');
      });
    }

    document.querySelectorAll('.ox-api-controls').forEach((controls) => {
      const targetSelector = controls.getAttribute('data-ox-api-target');
      if (!targetSelector) return;

      controls.querySelectorAll('[data-ox-api-toggle]').forEach((button) => {
        button.addEventListener('click', () => {
          const shouldOpen = button.getAttribute('data-ox-api-toggle') === 'expand';
          document.querySelectorAll(targetSelector).forEach((entry) => {
            if (entry instanceof HTMLDetailsElement) {
              entry.open = shouldOpen;
            }
          });
        });
      });
    });

    // Search functionality
    const searchButton = document.querySelector('.search-button');
    const searchOverlay = document.querySelector('.search-modal-overlay');
    const searchInput = document.querySelector('.search-input');
    const searchResults = document.querySelector('.search-results');
    const searchClose = document.querySelector('.search-close');
    let searchIndex = null;
    let selectedIndex = 0;
    let results = [];

    const openSearch = () => {
      searchOverlay.classList.add('open');
      searchInput.focus();
    };
    const closeSearch = () => {
      searchOverlay.classList.remove('open');
      searchInput.value = '';
      searchResults.innerHTML = '';
      selectedIndex = 0;
      results = [];
    };

    // Load search index
    const loadSearchIndex = async () => {
      if (searchIndex) return;
      try {
        const res = await fetch('{{base}}search-index.json');
        searchIndex = await res.json();
      } catch (e) {
        console.warn('Failed to load search index:', e);
      }
    };

    const parseScopedQuery = (query) => {
      const scopes = [];
      const terms = [];
      for (const part of query.trim().split(/\\s+/).filter(Boolean)) {
        if (part.startsWith('@') && part.length > 1) {
          scopes.push(part.slice(1).toLowerCase());
        } else {
          terms.push(part);
        }
      }
      return { text: terms.join(' ').trim(), scopes: [...new Set(scopes)] };
    };

    const getScopesForDoc = (doc) => {
      const source = (doc.id || doc.url || '').replace(/^\\/+/, '').toLowerCase();
      const segments = source.split('/').filter(Boolean);
      if (segments.length <= 1) return [];

      const scopes = [];
      let current = '';
      for (const segment of segments.slice(0, -1)) {
        current = current ? current + '/' + segment : segment;
        scopes.push(current);
      }
      return scopes;
    };

    const matchesScopes = (doc, scopes) => {
      if (!scopes.length) return true;
      const docScopes = new Set(getScopesForDoc(doc));
      return scopes.some((scope) => docScopes.has(scope));
    };

    // Tokenize query
    const tokenize = (text) => {
      const tokens = [];
      let current = '';
      for (const char of text) {
        const isCjk = /[\\u4E00-\\u9FFF\\u3400-\\u4DBF\\u3040-\\u309F\\u30A0-\\u30FF\\uAC00-\\uD7AF]/.test(char);
        if (isCjk) {
          if (current) { tokens.push(current.toLowerCase()); current = ''; }
          tokens.push(char);
        } else if (/[a-zA-Z0-9_]/.test(char)) {
          current += char;
        } else if (current) {
          tokens.push(current.toLowerCase());
          current = '';
        }
      }
      if (current) tokens.push(current.toLowerCase());
      return tokens;
    };

    // Perform search
    const performSearch = async (query) => {
      await loadSearchIndex();
      if (!searchIndex) {
        searchResults.innerHTML = '<div class="search-empty">Search index not available</div>';
        return;
      }

      const parsedQuery = parseScopedQuery(query);
      if (!parsedQuery.text && parsedQuery.scopes.length === 0) {
        searchResults.innerHTML = '';
        results = [];
        return;
      }

      const tokens = tokenize(parsedQuery.text);
      const k1 = 1.2, b = 0.75;
      const docScores = new Map();

      if (!tokens.length) {
        searchIndex.documents.forEach((doc, docIdx) => {
          if (matchesScopes(doc, parsedQuery.scopes)) {
            docScores.set(docIdx, { score: 0, matches: new Set() });
          }
        });
      }

      for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i];
        const isLast = i === tokens.length - 1;
        let matchingTerms = [];
        if (isLast && token.length >= 2) {
          matchingTerms = Object.keys(searchIndex.index).filter(t => t.startsWith(token));
        } else if (searchIndex.index[token]) {
          matchingTerms = [token];
        }

        for (const term of matchingTerms) {
          const postings = searchIndex.index[term] || [];
          const df = searchIndex.df[term] || 1;
          const idf = Math.log((searchIndex.doc_count - df + 0.5) / (df + 0.5) + 1.0);

          for (const posting of postings) {
            const doc = searchIndex.documents[posting.doc_idx];
            if (!doc) continue;
            if (!matchesScopes(doc, parsedQuery.scopes)) continue;
            const boost = posting.field === 'Title' ? 10 : posting.field === 'Heading' ? 5 : 1;
            const tf = posting.tf;
            const docLen = doc.body.length;
            const score = idf * ((tf * (k1 + 1)) / (tf + k1 * (1 - b + b * docLen / searchIndex.avg_dl))) * boost;

            if (!docScores.has(posting.doc_idx)) {
              docScores.set(posting.doc_idx, { score: 0, matches: new Set() });
            }
            const entry = docScores.get(posting.doc_idx);
            entry.score += score;
            entry.matches.add(term);
          }
        }
      }

      results = Array.from(docScores.entries())
        .map(([docIdx, data]) => {
          const doc = searchIndex.documents[docIdx];
          const scopes = getScopesForDoc(doc);
          let snippet = '';
          if (doc.body) {
            const bodyLower = doc.body.toLowerCase();
            let firstPos = -1;
            for (const match of data.matches) {
              const pos = bodyLower.indexOf(match);
              if (pos !== -1 && (firstPos === -1 || pos < firstPos)) firstPos = pos;
            }
            const start = firstPos === -1 ? 0 : Math.max(0, firstPos - 50);
            const end = Math.min(doc.body.length, start + 150);
            snippet = doc.body.slice(start, end);
            if (start > 0) snippet = '...' + snippet;
            if (end < doc.body.length) snippet += '...';
          }
          return { ...doc, score: data.score, scopes, snippet };
        })
        .sort((a, b) => b.score - a.score || a.title.localeCompare(b.title))
        .slice(0, 10);

      selectedIndex = 0;
      renderResults();
    };

    const renderResults = () => {
      if (!results.length) {
        searchResults.innerHTML = '<div class="search-empty">No results found</div>';
        return;
      }
      searchResults.innerHTML = results.map((r, i) =>
        '<a href="' + r.url + '" class="search-result' + (i === selectedIndex ? ' selected' : '') + '">' +
        '<div class="search-result-title">' + r.title + (r.scopes?.length ? '<span class="search-result-scope">@' + r.scopes[0] + '</span>' : '') + '</div>' +
        (r.snippet ? '<div class="search-result-snippet">' + r.snippet + '</div>' : '') +
        '</a>'
      ).join('');
    };

    // Event listeners
    if (searchButton) searchButton.addEventListener('click', openSearch);
    if (searchClose) searchClose.addEventListener('click', closeSearch);
    if (searchOverlay) searchOverlay.addEventListener('click', (e) => { if (e.target === searchOverlay) closeSearch(); });

    let searchTimeout = null;
    if (searchInput) {
      searchInput.addEventListener('input', () => {
        if (searchTimeout) clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => performSearch(searchInput.value), 150);
      });
      searchInput.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') closeSearch();
        else if (e.key === 'ArrowDown') {
          e.preventDefault();
          if (selectedIndex < results.length - 1) { selectedIndex++; renderResults(); }
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          if (selectedIndex > 0) { selectedIndex--; renderResults(); }
        } else if (e.key === 'Enter' && results[selectedIndex]) {
          e.preventDefault();
          window.location.href = results[selectedIndex].url;
        }
      });
    }

    // Global keyboard shortcut (/ or Cmd+K)
    document.addEventListener('keydown', (e) => {
      if ((e.key === '/' && !(e.target instanceof HTMLInputElement)) ||
          ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k')) {
        e.preventDefault();
        openSearch();
      }
    });
  </script>
</body>
</html>`;

/**
 * Bare HTML template (no navigation, no styles).
 */
export const BARE_HTML_TEMPLATE = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{title}}</title>
</head>
<body>
{{content}}
</body>
</html>`;

/**
 * Resolves SSG options with defaults.
 */
export function resolveSsgOptions(ssg: SsgOptions | boolean | undefined): ResolvedSsgOptions {
  if (ssg === false) {
    return {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    };
  }

  if (ssg === true || ssg === undefined) {
    return {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      theme: resolveTheme(undefined),
    };
  }

  return {
    enabled: ssg.enabled ?? true,
    extension: ssg.extension ?? ".html",
    clean: ssg.clean ?? false,
    bare: ssg.bare ?? false,
    siteName: ssg.siteName,
    ogImage: ssg.ogImage,
    generateOgImage: ssg.generateOgImage ?? false,
    lastUpdated: ssg.lastUpdated ?? false,
    siteUrl: ssg.siteUrl,
    theme: resolveTheme(ssg.theme),
  };
}

/**
 * Simple mustache-like template rendering.
 */
function renderTemplate(template: string, data: Record<string, unknown>): string {
  let result = template;

  // Handle conditionals: {{#key}}content{{/key}}
  result = result.replace(/\{\{#(\w+)\}\}([\s\S]*?)\{\{\/\1\}\}/g, (_, key, content) => {
    return data[key] ? content : "";
  });

  // Handle inverted conditionals: {{^key}}content{{/key}}
  result = result.replace(/\{\{\^(\w+)\}\}([\s\S]*?)\{\{\/\1\}\}/g, (_, key, content) => {
    return data[key] ? "" : content;
  });

  // Handle simple replacements: {{key}}
  result = result.replace(/\{\{(\w+)\}\}/g, (_, key) => {
    const value = data[key];
    if (value === undefined || value === null) return "";
    if (typeof value === "object") return JSON.stringify(value);
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") {
      return String(value);
    }
    return "";
  });

  return result;
}

/**
 * Extracts title from content or frontmatter.
 */
export function extractTitle(content: string, frontmatter: Record<string, unknown>): string {
  if (frontmatter.title && typeof frontmatter.title === "string") {
    return frontmatter.title;
  }

  const h1Match = content.match(/<h1[^>]*>([^<]+)<\/h1>/i);
  if (h1Match) {
    return h1Match[1].trim();
  }

  return "Untitled";
}

/**
 * Generates navigation HTML from nav groups.
 */
function _generateNavHtml(navGroups: NavGroup[], currentPath: string): string {
  return navGroups
    .map((group) => {
      const items = group.items
        .map((item) => {
          const isActive = item.path === currentPath;
          const activeClass = isActive ? " active" : "";
          return `              <li class="nav-item"><a href="${item.href}" class="nav-link${activeClass}">${item.title}</a></li>`;
        })
        .join("\n");

      return `          <div class="nav-section">
            <div class="nav-title">${group.title}</div>
            <ul class="nav-list">
${items}
            </ul>
          </div>`;
    })
    .join("\n");
}

/**
 * Generates TOC HTML from toc entries.
 */
function _generateTocHtml(toc: TocEntry[]): string {
  const flattenToc = (entries: TocEntry[], depth = 1): string[] => {
    const items: string[] = [];
    for (const entry of entries) {
      items.push(
        `        <li class="toc-item"><a href="#${entry.slug}" class="toc-link" style="--depth: ${depth}">${entry.text}</a></li>`,
      );
      if (entry.children && entry.children.length > 0) {
        items.push(...flattenToc(entry.children, depth + 1));
      }
    }
    return items;
  };
  return flattenToc(toc).join("\n");
}

/**
 * Generates bare HTML page (no navigation, no styles).
 */
export function generateBareHtmlPage(content: string, title: string): string {
  return renderTemplate(BARE_HTML_TEMPLATE, {
    title,
    content,
  });
}

/**
 * Generates HTML page with navigation using Rust NAPI bindings.
 */
export async function generateHtmlPage(
  pageData: SsgPageData,
  navGroups: NavGroup[],
  siteName: string,
  base: string,
  ogImage?: string,
  theme?: ResolvedThemeConfig,
): Promise<string> {
  const mod = await importNapiModule();

  // Convert TocEntry to the format expected by Rust
  const tocForRust = pageData.toc.map((entry) => ({
    depth: entry.depth,
    text: entry.text,
    slug: entry.slug,
  }));

  // Convert NavGroup to the format expected by Rust
  const toRustNavItem = (item: SsgNavItem): SsgNavItem => ({
    title: item.title,
    path: item.path,
    href: item.href,
    children: item.children?.map(toRustNavItem),
    collapsed: item.collapsed,
  });

  const navGroupsForRust = navGroups.map((group) => ({
    title: group.title,
    collapsed: group.collapsed,
    items: group.items.map(toRustNavItem),
  }));

  // Convert theme to NAPI format if provided
  const themeForRust = theme ? themeToNapi(theme) : undefined;

  // Convert entry page to NAPI format if provided
  const entryPageForRust = pageData.entryPage
    ? {
        hero: pageData.entryPage.hero
          ? {
              name: pageData.entryPage.hero.name,
              text: pageData.entryPage.hero.text,
              tagline: pageData.entryPage.hero.tagline,
              notice: pageData.entryPage.hero.notice
                ? {
                    title: pageData.entryPage.hero.notice.title,
                    body: pageData.entryPage.hero.notice.body,
                  }
                : undefined,
              image: pageData.entryPage.hero.image
                ? {
                    src: pageData.entryPage.hero.image.src,
                    lightSrc: pageData.entryPage.hero.image.lightSrc,
                    darkSrc: pageData.entryPage.hero.image.darkSrc,
                    alt: pageData.entryPage.hero.image.alt,
                    width: pageData.entryPage.hero.image.width,
                    height: pageData.entryPage.hero.image.height,
                  }
                : undefined,
              actions: pageData.entryPage.hero.actions?.map((a) => ({
                theme: a.theme,
                text: a.text,
                link: a.link,
              })),
            }
          : undefined,
        features: pageData.entryPage.features?.map((f) => ({
          icon: f.icon,
          title: f.title,
          details: f.details,
          link: f.link,
          linkText: f.linkText,
        })),
      }
    : undefined;

  return mod.generateSsgHtml(
    {
      title: pageData.title,
      description: pageData.description,
      content: pageData.content,
      toc: tocForRust,
      lastUpdated: pageData.lastUpdated,
      path: pageData.path,
      entryPage: entryPageForRust,
    },
    navGroupsForRust,
    {
      siteName,
      base,
      ogImage,
      theme: themeForRust,
    },
  );
}

interface GeneratedHtmlPage {
  inputPath: string;
  outputPath: string;
  html: string;
}

interface SharedAssetChunk {
  outputPath: string;
  publicPath: string;
  content: string;
}

interface CssSection {
  name: string;
  content: string;
}

const SSG_STYLE_BLOCK_RE =
  /[ \t]*<!-- ox-content:styles:start -->\s*<style>([\s\S]*?)<\/style>\s*<!-- ox-content:styles:end -->/;
const SSG_SCRIPT_BLOCK_RE =
  /[ \t]*<!-- ox-content:scripts:start -->\s*<script>([\s\S]*?)<\/script>\s*<!-- ox-content:scripts:end -->/;
const FIRST_INLINE_STYLE_RE = /[ \t]*<style>([\s\S]*?)<\/style>/;
const LAST_INLINE_BODY_SCRIPT_RE = /[ \t]*<script>([\s\S]*?)<\/script>\s*<\/body>/;
const CSS_SECTION_RE =
  /\/\* ox-content:css:([a-z0-9-]+):start \*\/\s*([\s\S]*?)\s*\/\* ox-content:css:\1:end \*\//g;
const SEARCH_CHUNK_RE = /\/\/ ox-content:search:start\s*([\s\S]*?)\s*\/\/ ox-content:search:end/;
const SEARCH_CHUNK_PLACEHOLDER = "__OX_CONTENT_SEARCH_CHUNK__";
const CORE_CSS_SECTION_NAMES = new Set(["base", "footer"]);
const THEME_INLINE_CSS_MAX_BYTES = 2048;

function createContentHash(content: string): string {
  return createHash("sha256").update(content).digest("hex").slice(0, 10);
}

function sanitizeChunkLabel(label: string): string {
  return (
    label
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "asset"
  );
}

function toPublicAssetPath(base: string, fileName: string): string {
  const normalizedBase = base.endsWith("/") ? base : `${base}/`;
  return `${normalizedBase}assets/${fileName}`;
}

function hasRelativeCssUrls(css: string): boolean {
  let cursor = 0;

  while (cursor < css.length) {
    const urlIndex = css.indexOf("url(", cursor);
    if (urlIndex === -1) {
      return false;
    }

    let valueStart = urlIndex + 4;
    while (valueStart < css.length && /\s/.test(css[valueStart])) {
      valueStart++;
    }

    const quote = css[valueStart] === '"' || css[valueStart] === "'" ? css[valueStart] : "";
    if (quote) {
      valueStart++;
    }

    let valueEnd = valueStart;
    while (valueEnd < css.length) {
      const char = css[valueEnd];
      if (quote) {
        if (char === "\\") {
          valueEnd += 2;
          continue;
        }
        if (char === quote) {
          break;
        }
      } else if (char === ")") {
        break;
      }
      valueEnd++;
    }

    const value = css.slice(valueStart, valueEnd).trim();
    if (
      value &&
      !value.startsWith("data:") &&
      !value.startsWith("http:") &&
      !value.startsWith("https:") &&
      !value.startsWith("//") &&
      !value.startsWith("/") &&
      !value.startsWith("#") &&
      !value.startsWith("blob:") &&
      !value.startsWith("var(")
    ) {
      return true;
    }

    cursor = valueEnd + 1;
  }

  return false;
}

function createSharedAssetChunk(
  type: "css" | "js",
  label: string,
  content: string,
  outDir: string,
  base: string,
): SharedAssetChunk {
  const hash = createContentHash(content);
  const fileName = `ox-content-${sanitizeChunkLabel(label)}-${hash}.${type}`;

  return {
    outputPath: path.join(outDir, "assets", fileName),
    publicPath: toPublicAssetPath(base, fileName),
    content,
  };
}

function extractCssSections(cssContent: string): CssSection[] {
  return Array.from(cssContent.matchAll(CSS_SECTION_RE))
    .map(([, name, content]) => ({
      name,
      content: content.trim(),
    }))
    .filter((section) => section.content.length > 0);
}

function getOrCreateSharedChunk(
  chunks: Map<string, SharedAssetChunk>,
  type: "css" | "js",
  label: string,
  content: string,
  outDir: string,
  base: string,
): SharedAssetChunk {
  let chunk = chunks.get(content);
  if (!chunk) {
    chunk = createSharedAssetChunk(type, label, content, outDir, base);
    chunks.set(content, chunk);
  }
  return chunk;
}

function buildStyleReplacement(
  cssContent: string,
  cssChunks: Map<string, SharedAssetChunk>,
  outDir: string,
  base: string,
): string {
  const sections = extractCssSections(cssContent);
  const effectiveSections =
    sections.length > 0
      ? sections
      : [
          {
            name: "css",
            content: cssContent.trim(),
          },
        ];

  const coreContent = effectiveSections
    .filter((section) => CORE_CSS_SECTION_NAMES.has(section.name))
    .map((section) => section.content)
    .join("\n")
    .trim();
  const fragments: string[] = [];

  if (coreContent) {
    const coreChunk = getOrCreateSharedChunk(cssChunks, "css", "core", coreContent, outDir, base);
    fragments.push(`  <link rel="stylesheet" href="${coreChunk.publicPath}">`);
  }

  for (const section of effectiveSections) {
    if (CORE_CSS_SECTION_NAMES.has(section.name)) {
      continue;
    }

    const shouldInlineTheme =
      section.name === "theme" &&
      (hasRelativeCssUrls(section.content) || section.content.length <= THEME_INLINE_CSS_MAX_BYTES);
    if (shouldInlineTheme || hasRelativeCssUrls(section.content)) {
      fragments.push(`  <style>${section.content}</style>`);
      continue;
    }

    const chunk = getOrCreateSharedChunk(
      cssChunks,
      "css",
      section.name,
      section.content,
      outDir,
      base,
    );
    fragments.push(`  <link rel="stylesheet" href="${chunk.publicPath}">`);
  }

  return fragments.join("\n");
}

function buildScriptReplacement(
  jsContent: string,
  jsChunks: Map<string, SharedAssetChunk>,
  outDir: string,
  base: string,
): string {
  const searchMatch = jsContent.match(SEARCH_CHUNK_RE);

  if (searchMatch && jsContent.includes(SEARCH_CHUNK_PLACEHOLDER)) {
    const searchContent = searchMatch[1].trim();
    if (searchContent) {
      const searchChunk = getOrCreateSharedChunk(
        jsChunks,
        "js",
        "search",
        searchContent,
        outDir,
        base,
      );
      const coreContent = jsContent
        .replace(SEARCH_CHUNK_RE, "")
        .replaceAll(SEARCH_CHUNK_PLACEHOLDER, searchChunk.publicPath)
        .trim();

      if (coreContent) {
        const coreChunk = getOrCreateSharedChunk(jsChunks, "js", "core", coreContent, outDir, base);
        return `  <script defer src="${coreChunk.publicPath}"></script>`;
      }
    }
  }

  const fallbackContent = jsContent.trim();
  if (!fallbackContent) {
    return "";
  }

  const chunk = getOrCreateSharedChunk(jsChunks, "js", "js", fallbackContent, outDir, base);
  return `  <script defer src="${chunk.publicPath}"></script>`;
}

async function externalizeSharedPageAssets(
  pages: GeneratedHtmlPage[],
  outDir: string,
  base: string,
): Promise<{ pages: GeneratedHtmlPage[]; assets: string[] }> {
  const cssChunks = new Map<string, SharedAssetChunk>();
  const jsChunks = new Map<string, SharedAssetChunk>();

  const optimizedPages = pages.map((page) => {
    let html = page.html;

    const styleMatch = html.match(SSG_STYLE_BLOCK_RE);
    if (styleMatch) {
      const replacement = buildStyleReplacement(styleMatch[1], cssChunks, outDir, base);
      html = html.replace(SSG_STYLE_BLOCK_RE, replacement);
    } else {
      const inlineStyleMatch = html.match(FIRST_INLINE_STYLE_RE);
      if (inlineStyleMatch) {
        const replacement = buildStyleReplacement(inlineStyleMatch[1], cssChunks, outDir, base);
        html = html.replace(FIRST_INLINE_STYLE_RE, replacement);
      }
    }

    const scriptMatch = html.match(SSG_SCRIPT_BLOCK_RE);
    if (scriptMatch) {
      const replacement = buildScriptReplacement(scriptMatch[1], jsChunks, outDir, base);
      html = html.replace(SSG_SCRIPT_BLOCK_RE, replacement);
    } else {
      const inlineScriptMatch = html.match(LAST_INLINE_BODY_SCRIPT_RE);
      if (inlineScriptMatch) {
        const replacement = buildScriptReplacement(inlineScriptMatch[1], jsChunks, outDir, base);
        html = html.replace(
          LAST_INLINE_BODY_SCRIPT_RE,
          replacement ? `${replacement}\n</body>` : "</body>",
        );
      }
    }

    return {
      ...page,
      html,
    };
  });

  const chunks = [...cssChunks.values(), ...jsChunks.values()];
  await Promise.all(
    chunks.map(async (chunk) => {
      await fs.mkdir(path.dirname(chunk.outputPath), { recursive: true });
      await fs.writeFile(chunk.outputPath, chunk.content, "utf-8");
    }),
  );

  return {
    pages: optimizedPages,
    assets: chunks.map((chunk) => chunk.outputPath),
  };
}

/**
 * Converts a markdown file path to its corresponding HTML output path.
 */
export function getOutputPath(
  inputPath: string,
  srcDir: string,
  outDir: string,
  extension: string,
): string {
  const relativePath = path.relative(srcDir, inputPath);
  const baseName = relativePath.replace(/\.(?:md|markdown)$/i, extension);

  if (baseName.endsWith(`index${extension}`)) {
    return path.join(outDir, baseName);
  }

  const dirName = baseName.replace(new RegExp(`\\${extension}$`), "");
  return path.join(outDir, dirName, `index${extension}`);
}

/**
 * Converts a markdown file path to a relative URL path.
 */
export function getUrlPath(inputPath: string, srcDir: string): string {
  const relativePath = path.relative(srcDir, inputPath);
  const baseName = relativePath.replace(/\.(?:md|markdown)$/i, "");

  if (baseName === "index" || baseName.endsWith("/index")) {
    return baseName.replace(/\/?index$/, "") || "/";
  }

  return baseName;
}

/**
 * Converts a markdown file path to an href.
 */
export function getHref(
  inputPath: string,
  srcDir: string,
  base: string,
  extension: string,
): string {
  const urlPath = getUrlPath(inputPath, srcDir);
  if (urlPath === "/" || urlPath === "") {
    return `${base}index${extension}`;
  }
  return `${base}${urlPath}/index${extension}`;
}

/**
 * Gets the OG image output path for a given markdown file.
 */
function getOgImagePath(inputPath: string, srcDir: string, outDir: string): string {
  const relativePath = path.relative(srcDir, inputPath);
  const baseName = relativePath.replace(/\.(?:md|markdown)$/i, "");

  if (baseName === "index" || baseName.endsWith("/index")) {
    const dirPath = baseName.replace(/\/?index$/, "") || "";
    return path.join(outDir, dirPath, "og-image.png");
  }

  return path.join(outDir, baseName, "og-image.png");
}

/**
 * Gets the OG image URL for use in meta tags.
 * If siteUrl is provided, returns an absolute URL (required for SNS sharing).
 */
function getOgImageUrl(inputPath: string, srcDir: string, base: string, siteUrl?: string): string {
  const urlPath = getUrlPath(inputPath, srcDir);
  let relativePath: string;
  if (urlPath === "/" || urlPath === "") {
    relativePath = `${base}og-image.png`;
  } else {
    relativePath = `${base}${urlPath}/og-image.png`;
  }

  // Return absolute URL if siteUrl is provided
  if (siteUrl) {
    const cleanSiteUrl = siteUrl.replace(/\/$/, "");
    return `${cleanSiteUrl}${relativePath}`;
  }

  return relativePath;
}

/**
 * Gets display title from file path.
 */
function getDisplayTitle(filePath: string): string {
  const fileName = path.basename(filePath, path.extname(filePath));

  if (fileName === "index") {
    const dirName = path.basename(path.dirname(filePath));
    if (dirName && dirName !== ".") {
      return formatTitle(dirName);
    }
    return "Home";
  }

  return formatTitle(fileName);
}

/**
 * Formats a file/dir name as a title.
 */
export function formatTitle(name: string): string {
  return name
    .replace(/[-_]([a-z])/g, (_, char) => " " + char.toUpperCase())
    .replace(/^[a-z]/, (char) => char.toUpperCase());
}

/**
 * Collects all markdown files from the source directory.
 */
export async function collectMarkdownFiles(srcDir: string): Promise<string[]> {
  const pattern = path.join(srcDir, "**/*.{md,markdown}");
  const files = await glob(pattern, {
    nodir: true,
    ignore: ["**/node_modules/**", "**/dist/**", "**/.git/**"],
  });
  return files.sort();
}

/**
 * Navigation group for hierarchical navigation.
 */
export interface NavGroup {
  title: string;
  items: SsgNavItem[];
  collapsed?: boolean;
}

/**
 * Builds navigation items from markdown files, grouped by directory.
 */
export function buildNavItems(
  markdownFiles: string[],
  srcDir: string,
  base: string,
  extension: string,
): NavGroup[] {
  const groups = new Map<string, SsgNavItem[]>();

  // Define the order of groups (api at the bottom)
  const groupOrder = ["", "examples", "packages", "api"];

  for (const file of markdownFiles) {
    const relativePath = path.relative(srcDir, file);
    const parts = relativePath.split(path.sep);

    // Determine group: first directory or '' for root files
    let groupKey = "";
    if (parts.length > 1) {
      groupKey = parts[0];
    }

    if (!groups.has(groupKey)) {
      groups.set(groupKey, []);
    }

    const urlPath = getUrlPath(file, srcDir);

    // Use "Overview" for root index.md, otherwise use getDisplayTitle
    let title: string;
    if (urlPath === "/" || urlPath === "") {
      title = "Overview";
    } else {
      title = getDisplayTitle(file);
    }

    groups.get(groupKey)!.push({
      title,
      path: urlPath,
      href: getHref(file, srcDir, base, extension),
    });
  }

  // Sort items within each group: index files first, then alphabetically
  const sortItems = (items: SsgNavItem[]) => {
    return items.sort((a, b) => {
      // Root index (Overview) comes first
      const aIsRoot = a.path === "/" || a.path === "";
      const bIsRoot = b.path === "/" || b.path === "";
      if (aIsRoot && !bIsRoot) return -1;
      if (!aIsRoot && bIsRoot) return 1;
      // Otherwise, maintain alphabetical order by title
      return a.title.localeCompare(b.title);
    });
  };

  // Convert to array and sort by group order
  const result: NavGroup[] = [];

  for (const key of groupOrder) {
    const items = groups.get(key);
    if (items && items.length > 0) {
      result.push({
        title: key === "" ? "Guide" : formatTitle(key),
        items: sortItems(items),
      });
      groups.delete(key);
    }
  }

  // Add any remaining groups
  for (const [key, items] of groups) {
    if (items.length > 0) {
      result.push({
        title: formatTitle(key),
        items: sortItems(items),
      });
    }
  }

  return result;
}

function isSafeSidebarLink(link: string): boolean {
  const trimmed = link.trim();
  if (trimmed.startsWith("//")) {
    return false;
  }
  return !/^[a-z][a-z0-9+.-]*:/i.test(trimmed) || /^(https?:|mailto:)/i.test(trimmed);
}

function sidebarPath(link?: string): string {
  if (!link || !isSafeSidebarLink(link)) {
    return "";
  }
  if (/^(https?:|mailto:|#)/i.test(link.trim())) {
    return "";
  }
  const withoutHash = link.trim().split("#", 1)[0].split("?", 1)[0];
  const bare = withoutHash
    .replace(/^\/+/, "")
    .replace(/\/$/, "")
    .replace(/\.(md|markdown)$/i, "");
  if (!bare || bare === "index") {
    return "/";
  }
  return bare.replace(/\/index$/, "");
}

function sidebarHref(link: string | undefined, base: string, extension: string): string {
  if (!link) {
    return "#";
  }
  const trimmed = link.trim();
  if (!isSafeSidebarLink(trimmed)) {
    return "#";
  }
  if (/^(https?:|mailto:|#)/i.test(trimmed)) {
    return trimmed;
  }
  const hash = trimmed.includes("#") ? `#${trimmed.split("#").slice(1).join("#")}` : "";
  const withoutHash = trimmed.split("#", 1)[0].replace(/^\/+/, "").replace(/\/$/, "");
  const withoutExt = withoutHash.replace(/\.(md|markdown)$/i, "");
  const route =
    !withoutExt || withoutExt === "index" ? "index" : `${withoutExt.replace(/\/index$/, "")}/index`;
  return `${base}${route}${extension}${hash}`;
}

/**
 * Builds navigation items from an explicit theme sidebar tree.
 */
export function buildThemeNavItems(
  sidebar: SidebarItem[],
  base: string,
  extension: string,
): NavGroup[] {
  const toNavItem = (item: SidebarItem): SsgNavItem => {
    const navItem: SsgNavItem = {
      title: item.text ?? item.link ?? "Untitled",
      path: sidebarPath(item.link),
      href: sidebarHref(item.link, base, extension),
    };
    if (item.items?.length) {
      navItem.children = item.items.map(toNavItem);
    }
    if (item.collapsed !== undefined) {
      navItem.collapsed = item.collapsed;
    }
    return navItem;
  };
  const groups: NavGroup[] = [];
  let looseItems: SsgNavItem[] = [];
  const flushLooseItems = () => {
    if (looseItems.length > 0) {
      groups.push({ title: "Guide", items: looseItems });
      looseItems = [];
    }
  };

  for (const item of sidebar) {
    if (item.items?.length && !item.link) {
      flushLooseItems();
      groups.push({
        title: item.text ?? "Guide",
        items: item.items.map(toNavItem),
        collapsed: item.collapsed,
      });
    } else {
      looseItems.push(toNavItem(item));
    }
  }
  flushLooseItems();
  return groups;
}

/**
 * Builds all markdown files to static HTML.
 */
export async function buildSsg(
  options: ResolvedOptions,
  root: string,
): Promise<{ files: string[]; errors: string[] }> {
  const ssgOptions = options.ssg;
  if (!ssgOptions.enabled) {
    return { files: [], errors: [] };
  }

  const srcDir = path.resolve(root, options.srcDir);
  const outDir = path.resolve(root, options.outDir);
  const base = options.base.endsWith("/") ? options.base : options.base + "/";
  const generatedFiles: string[] = [];
  const generatedPages: GeneratedHtmlPage[] = [];
  const errors: string[] = [];

  // Clean output directory if requested
  if (ssgOptions.clean) {
    try {
      await fs.rm(outDir, { recursive: true, force: true });
    } catch {
      // Ignore if directory doesn't exist
    }
  }

  // Collect markdown files
  const markdownFiles = await collectMarkdownFiles(srcDir);

  // Build navigation
  const navItems = ssgOptions.theme?.sidebar.length
    ? buildThemeNavItems(ssgOptions.theme.sidebar, base, ssgOptions.extension)
    : buildNavItems(markdownFiles, srcDir, base, ssgOptions.extension);

  // Get site name from options or package.json
  let siteName = ssgOptions.siteName ?? "Documentation";
  if (!ssgOptions.siteName) {
    try {
      const pkgPath = path.join(root, "package.json");
      const pkg = JSON.parse(await fs.readFile(pkgPath, "utf-8"));
      if (pkg.name) {
        siteName = formatTitle(pkg.name);
      }
    } catch {
      // Use default
    }
  }

  // Collect OG image entries for batch rendering
  const ogImageEntries: OgImagePageEntry[] = [];
  // Parallel array tracking the inputPath for each ogImageEntry (same index)
  const ogImageInputPaths: string[] = [];
  // Map from inputPath to OG image URL (filled after batch render)
  const ogImageUrlMap = new Map<string, string>();

  // Determine if OG images should be generated
  const shouldGenerateOgImages =
    (options.ogImage || ssgOptions.generateOgImage) && !ssgOptions.bare;

  // Collect page metadata for OG image generation
  interface PageProcessResult {
    inputPath: string;
    transformedHtml: string;
    title: string;
    description?: string;
    lastUpdated?: number;
    frontmatter: Record<string, unknown>;
    toc: TocEntry[];
  }
  const pageResults: PageProcessResult[] = [];
  const napi = ssgOptions.lastUpdated ? await importNapiModule() : undefined;

  // Process each file: transform markdown and collect metadata
  for (const inputPath of markdownFiles) {
    try {
      const content = await fs.readFile(inputPath, "utf-8");
      // Pass SSG options to transform for .md -> .html link conversion in Rust
      // The sourcePath is used to determine if the file is an index file for correct relative link resolution
      const result = await transformMarkdown(content, inputPath, options, {
        convertMdLinks: true,
        baseUrl: base,
        sourcePath: inputPath,
      });

      // Apply built-in plugin transformations (No-JS First)
      let transformedHtml = result.html;

      // Protect mermaid SVGs from rehype processing in plugins
      const { html: protectedHtml, svgs: mermaidSvgs } = protectMermaidSvgs(transformedHtml);
      transformedHtml = protectedHtml;

      // Transform Tabs, YouTube, GitHub, OGP, Mermaid plugins
      const pluginOptions: TransformAllOptions = {
        tabs: true,
        youtube: true,
        github: true,
        ogp: true,
        mermaid: true,
        githubToken: process.env.GITHUB_TOKEN,
      };
      transformedHtml = await transformAllPlugins(transformedHtml, pluginOptions);

      // Transform Island components
      if (hasIslands(transformedHtml)) {
        const islandResult = await transformIslands(transformedHtml);
        transformedHtml = islandResult.html;
      }

      // Restore protected mermaid SVGs
      transformedHtml = restoreMermaidSvgs(transformedHtml, mermaidSvgs);

      const title = extractTitle(transformedHtml, result.frontmatter);
      const description = result.frontmatter.description as string | undefined;

      pageResults.push({
        inputPath,
        transformedHtml,
        title,
        description,
        lastUpdated: napi?.getGitLastUpdated(inputPath, root) ?? undefined,
        frontmatter: result.frontmatter,
        toc: result.toc,
      });

      // Collect OG image entry if generation is enabled
      if (shouldGenerateOgImages) {
        const ogImageOutputPath = getOgImagePath(inputPath, srcDir, outDir);
        const { layout: _layout, ...frontmatterRest } = result.frontmatter;
        ogImageEntries.push({
          props: {
            ...frontmatterRest,
            title,
            description,
            siteName,
          },
          outputPath: ogImageOutputPath,
        });
        ogImageInputPaths.push(inputPath);
        // Pre-compute URL so HTML can reference it
        ogImageUrlMap.set(inputPath, getOgImageUrl(inputPath, srcDir, base, ssgOptions.siteUrl));
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      errors.push(`Failed to process ${inputPath}: ${errorMessage}`);
    }
  }

  // Batch generate OG images (Chromium-based)
  if (shouldGenerateOgImages && ogImageEntries.length > 0) {
    try {
      const ogResults = await generateOgImages(ogImageEntries, options.ogImageOptions, root);
      let ogSuccessCount = 0;
      for (let i = 0; i < ogResults.length; i++) {
        const result = ogResults[i];
        if (result.error) {
          errors.push(`OG image failed for ${result.outputPath}: ${result.error}`);
          // Remove failed entries so og:image / twitter:image meta tags are not emitted
          ogImageUrlMap.delete(ogImageInputPaths[i]);
        } else {
          generatedFiles.push(result.outputPath);
          ogSuccessCount++;
        }
      }
      if (ogSuccessCount > 0) {
        const cachedCount = ogResults.filter((r) => r.cached && !r.error).length;
        console.log(
          `[ox-content:og-image] Generated ${ogSuccessCount} OG images` +
            (cachedCount > 0 ? ` (${cachedCount} from cache)` : ""),
        );
      }
    } catch (err) {
      // Non-fatal: OG image failures never block the SSG build
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.warn(`[ox-content:og-image] Batch generation failed: ${errorMessage}`);
      // Clear all entries so og:image / twitter:image meta tags are not emitted
      ogImageUrlMap.clear();
    }
  }

  // Generate HTML pages
  for (const pageResult of pageResults) {
    try {
      const { inputPath, transformedHtml, title, description, lastUpdated, frontmatter, toc } =
        pageResult;

      // Determine OG image URL for this page
      let pageOgImage = ssgOptions.ogImage; // fallback to static URL
      if (shouldGenerateOgImages && ogImageUrlMap.has(inputPath)) {
        pageOgImage = ogImageUrlMap.get(inputPath);
      }

      // Check if this is an entry page (layout: entry)
      let entryPage: SsgEntryPageConfig | undefined;
      if (frontmatter.layout === "entry") {
        entryPage = {
          hero: frontmatter.hero as HeroConfig | undefined,
          features: frontmatter.features as FeatureConfig[] | undefined,
        };
      }

      // Generate HTML based on bare option
      let html: string;
      if (ssgOptions.bare) {
        html = generateBareHtmlPage(transformedHtml, title);
      } else {
        const pageData: SsgPageData = {
          title,
          description,
          content: transformedHtml,
          toc,
          lastUpdated,
          frontmatter,
          path: getUrlPath(inputPath, srcDir),
          href: getHref(inputPath, srcDir, base, ssgOptions.extension),
          entryPage,
        };
        html = await generateHtmlPage(
          pageData,
          navItems,
          siteName,
          base,
          pageOgImage,
          ssgOptions.theme,
        );
      }

      const outputPath = getOutputPath(inputPath, srcDir, outDir, ssgOptions.extension);
      generatedPages.push({
        inputPath,
        outputPath,
        html,
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      errors.push(`Failed to generate HTML for ${pageResult.inputPath}: ${errorMessage}`);
    }
  }

  const optimizedOutput = await externalizeSharedPageAssets(generatedPages, outDir, base);
  generatedFiles.push(...optimizedOutput.assets);

  for (const page of optimizedOutput.pages) {
    await fs.mkdir(path.dirname(page.outputPath), { recursive: true });
    await fs.writeFile(page.outputPath, page.html, "utf-8");
    generatedFiles.push(page.outputPath);
  }

  return { files: generatedFiles, errors };
}
