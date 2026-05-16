# nav-generator.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/nav-generator.ts)**

> 4 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="generatenavcode" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateNavCode(navItems: NavItem[], exportName: string = &quot;apiNav&quot;): string</code><span class="ox-api-entry__description">Generates TypeScript code for navigation metadata export. Creates a complete, s…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates TypeScript code for navigation metadata export. Creates a complete, self-contained TypeScript file that:</p>
<ul>
<li>Defines the NavItem interface</li>
<li>Exports navigation items as a const</li>
<li>Uses <code>as const</code> for type-safe literal types</li>
<li>Includes auto-generation notice The generated code is production-ready and suitable for direct import in Vue, React, or vanilla TypeScript applications.</li>
</ul>
<h2>Generated Code Example</h2>
<pre><code class="language-typescript">export interface NavItem {
title: string;
path: string;
children?: NavItem[];
}
export const apiNav: NavItem[] = [
{ &quot;title&quot;: &quot;Docs&quot;, &quot;path&quot;: &quot;/api/docs&quot; },
{ &quot;title&quot;: &quot;Transform&quot;, &quot;path&quot;: &quot;/api/transform&quot; },
// ...
] as const;</code></pre>
<h2>Features</h2>
<ul>
<li><strong>Type Safety</strong>: Includes NavItem interface definition</li>
<li><strong>Readonly</strong>: Uses <code>as const</code> to ensure immutability</li>
<li><strong>IDE Support</strong>: Full IntelliSense and autocomplete</li>
<li><strong>Self-Documenting</strong>: Includes notice that file is auto-generated Use custom names for different navigation sections ready to write to a .ts file</li>
</ul>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/nav-generator.ts#L246-L262">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">navItems</code>
    <code class="ox-api-entry__param-type">NavItem[]</code>
  </div>
  <p class="ox-api-entry__param-description">Array of navigation items to export</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">exportName</code>
    <code class="ox-api-entry__param-type">unknown</code>
  </div>
  <p class="ox-api-entry__param-description">Name of the exported const (default: &#39;apiNav&#39;)</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">Complete TypeScript source code as string,</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<pre><code class="language-ts">const navItems = [
  { title: &#39;Home&#39;, path: &#39;/api/index&#39; },
  { title: &#39;Transform&#39;, path: &#39;/api/transform&#39; },
];
const code = generateNavCode(navItems, &#39;apiNav&#39;);
await fs.promises.writeFile(&#39;docs/api/nav.ts&#39;, code, &#39;utf-8&#39;);</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@see</span><span class="ox-api-entry__tag-value">generateNavMetadata For generating NavItem arrays from extracted docs</span></li></ul>
</div>
  </div>
</details>

<details id="generatenavmetadata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateNavMetadata(docs: ExtractedDocs[], basePath: string = &quot;/api&quot;): NavItem[]</code><span class="ox-api-entry__description">Generates sidebar navigation metadata from extracted documentation. Takes an ar…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates sidebar navigation metadata from extracted documentation. Takes an array of extracted documentation and produces a flat navigation structure suitable for sidebar menus. Items are:</p>
<ul>
<li>Sorted alphabetically by display name</li>
<li>Formatted with readable titles</li>
<li>Prefixed with the specified base path</li>
</ul>
<h2>Naming Conventions</h2>
<ul>
<li><code>transform.ts</code> → <code>{ title: &#39;Transform&#39;, path: &#39;/api/transform&#39; }</code></li>
<li><code>nav-generator.ts</code> → <code>{ title: &#39;Nav Generator&#39;, path: &#39;/api/nav-generator&#39; }</code></li>
<li><code>index.ts</code> or <code>index-module.ts</code> → <code>{ title: &#39;Overview&#39;, path: &#39;/api/index&#39; }</code></li>
<li><code>types.ts</code> → <code>{ title: &#39;Types&#39;, path: &#39;/api/types&#39; }</code></li>
</ul>
<h2>Sorting</h2>
<p>Items are sorted alphabetically by display title for consistent ordering. Special item &#39;Overview&#39; sorts naturally with others (O comes after most letters).</p>
<h2>Path Generation</h2>
<p>The generated paths are used to import corresponding Markdown files:</p>
<ul>
<li>Path <code>/api/transform</code> → Import from <code>../api/transform.md</code></li>
<li>Path <code>/api/index</code> → Import from <code>../api/index.md</code> Use &#39;/api&#39; for main API docs, &#39;/helpers&#39; for utilities, etc.</li>
</ul>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/nav-generator.ts#L116-L128">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type">ExtractedDocs[]</code>
  </div>
  <p class="ox-api-entry__param-description">Array of extracted documentation (file + entries)</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">basePath</code>
    <code class="ox-api-entry__param-type">unknown</code>
  </div>
  <p class="ox-api-entry__param-description">Base path prefix for navigation URLs (default: &#39;/api&#39;)</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">NavItem[]</code>
  <p class="ox-api-entry__return-description">Array of navigation items ready to use or export to TypeScript</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<pre><code class="language-ts">const navItems = generateNavMetadata(
  [
    { file: &#39;transform.ts&#39;, entries: [...] },
    { file: &#39;docs.ts&#39;, entries: [...] },
    { file: &#39;types.ts&#39;, entries: [...] },
  ],
  &#39;/api&#39;
);
// Returns:
// [
//   { title: &#39;Docs&#39;, path: &#39;/api/docs&#39; },
//   { title: &#39;Transform&#39;, path: &#39;/api/transform&#39; },
//   { title: &#39;Types&#39;, path: &#39;/api/types&#39; },
// ]</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@see</span><span class="ox-api-entry__tag-value">generateNavCode For converting these items to TypeScript code</span></li></ul>
</div>
  </div>
</details>

<details id="getdocdisplayname" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">getDocDisplayName(filePath: string): string</code><span class="ox-api-entry__description">Gets the human-readable display name for a documentation file. Transforms file…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the human-readable display name for a documentation file. Transforms file paths and names into proper title case:</p>
<ul>
<li>Extracts base name (e.g., &#39;transform.ts&#39; → &#39;transform&#39;)</li>
<li>Converts kebab-case to Title Case (e.g., &#39;nav-generator&#39; → &#39;Nav Generator&#39;)</li>
<li>Converts camelCase to Title Case (e.g., &#39;transformMarkdown&#39; → &#39;Transform Markdown&#39;)</li>
<li>Handles special cases (index → &#39;Overview&#39;)</li>
</ul>
<h2>Examples</h2>
<ul>
<li><code>&#39;/path/to/transform.ts&#39;</code> → <code>&#39;Transform&#39;</code></li>
<li><code>&#39;nav-generator.ts&#39;</code> → <code>&#39;Nav Generator&#39;</code></li>
<li><code>&#39;index.ts&#39;</code> → <code>&#39;Overview&#39;</code></li>
<li><code>&#39;index-module.ts&#39;</code> → <code>&#39;Overview&#39;</code></li>
<li><code>&#39;myFunction.ts&#39;</code> → <code>&#39;My Function&#39;</code> (with camelCase handling)</li>
</ul>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/nav-generator.ts#L152-L165">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">filePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Full or relative file path</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">Formatted display name suitable for UI labels</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@internal</span><span class="ox-api-entry__tag-value"></span></li></ul>
</div>
  </div>
</details>

<details id="getdocfilename" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">getDocFileName(filePath: string): string</code><span class="ox-api-entry__description">Gets the file name (without extension) for use in navigation paths. This handle…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the file name (without extension) for use in navigation paths. This handles filename conflicts that may occur during generation:</p>
<ul>
<li>Preserves most names as-is</li>
<li>Special handling for index files to maintain consistency</li>
</ul>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/nav-generator.ts#L179-L189">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">filePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Source file path</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">File name without extension, ready for URL paths</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@internal</span><span class="ox-api-entry__tag-value"></span></li></ul>
</div>
  </div>
</details>
