# transform.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts)**

> 11 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="buildtoctree" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">buildTocTree(entries: TocEntry[]): TocEntry[]</code><span class="ox-api-entry__description">Builds nested TOC tree from flat list.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Builds nested TOC tree from flat list.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L514-L534">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">entries</code>
    <code class="ox-api-entry__param-type">TocEntry[]</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">TocEntry[]</code>
  
</div>
</div>
  </div>
</details>

<details id="extractimports" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">extractImports(content: string): string[]</code><span class="ox-api-entry__description">Extracts imports from Markdown content. Supports importing components for inter…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extracts imports from Markdown content. Supports importing components for interactive islands.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L595-L605">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">content</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string[]</code>
  
</div>
</div>
  </div>
</details>

<details id="generatemodulecode" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateModuleCode(html: string, frontmatter: Record&lt;string, unknown&gt;, toc: TocEntry[], filePath: string, _options: ResolvedOptions): string</code><span class="ox-api-entry__description">Generates the JavaScript module code.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates the JavaScript module code.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L539-L588">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">frontmatter</code>
    <code class="ox-api-entry__param-type">Record</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">toc</code>
    <code class="ox-api-entry__param-type">TocEntry[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">filePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_options</code>
    <code class="ox-api-entry__param-type">ResolvedOptions</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  
</div>
</div>
  </div>
</details>

<details id="generateogimagesvg" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateOgImageSvg(data: OgImageData, config?: OgImageConfig): Promise&lt;string | null&gt;</code><span class="ox-api-entry__description">Generates an OG image SVG using the Rust-based generator. This function uses th…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates an OG image SVG using the Rust-based generator. This function uses the Rust NAPI bindings to generate SVG-based OG images for social media previews. The SVG can be served directly or converted to PNG/JPEG for broader compatibility. In the future, custom JS templates can be provided to override the default Rust-based template.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L621-L643">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">data</code>
    <code class="ox-api-entry__param-type">OgImageData</code>
  </div>
  <p class="ox-api-entry__param-description">OG image data (title, description, etc.)</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type">OgImageConfig</code>
  </div>
  <p class="ox-api-entry__param-description">Optional OG image configuration — optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  <p class="ox-api-entry__return-description">SVG string or null if NAPI bindings are unavailable</p>
</div>
</div>
  </div>
</details>

<details id="jstransformoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JsTransformOptions</code><span class="ox-api-entry__description">Options for Rust-based Markdown transformation. Controls which Markdown extensi…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for Rust-based Markdown transformation. Controls which Markdown extensions and features are enabled during parsing and rendering.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L136-L226">View source</a></p>
  </div>
</details>

<details id="loadnapibindings" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">loadNapiBindings(): Promise&lt;NapiBindings | null&gt;</code><span class="ox-api-entry__description">Lazily loads and caches NAPI bindings. This function uses lazy loading to defer…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Lazily loads and caches NAPI bindings. This function uses lazy loading to defer the import of NAPI bindings until they&#39;re actually needed. The bindings are loaded only once and cached for subsequent uses. If loading fails (e.g., bindings not built), the failure is cached to avoid repeated load attempts.</p>
<h2>Performance Considerations</h2>
<p>The first call to this function may have a slight performance penalty due to module loading. Subsequent calls use the cached result and are essentially zero-cost.</p>
<h2>Error Handling</h2>
<p>If NAPI bindings are not available (not built, wrong architecture, etc.), this function returns <code>null</code>. The caller should handle this gracefully or provide fallback behavior.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L279-L302">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  <p class="ox-api-entry__return-description">Promise resolving to NAPI bindings or null if unavailable</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<pre><code class="language-ts">// Simple check with fallback
const napi = await loadNapiBindings();
if (!napi) {
  console.warn(&#39;NAPI bindings not available, using fallback&#39;);
  return fallbackRender(content);
}
// Use Rust implementation
const result = napi.transform(content, { gfm: true });</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@internal</span><span class="ox-api-entry__tag-value"></span></li></ul>
</div>
  </div>
</details>

<details id="napibindings" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiBindings</code><span class="ox-api-entry__description">NAPI bindings for Rust-based Markdown processing. Provides access to compiled R…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI bindings for Rust-based Markdown processing. Provides access to compiled Rust functions for high-performance Markdown parsing and rendering operations.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L47-L96">View source</a></p>
  </div>
</details>

<details id="ogimageconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageConfig</code><span class="ox-api-entry__description">OG image configuration.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image configuration.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L115-L128">View source</a></p>
  </div>
</details>

<details id="ogimagedata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageData</code><span class="ox-api-entry__description">OG image data for generating social media preview images.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image data for generating social media preview images.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L101-L110">View source</a></p>
  </div>
</details>

<details id="parsefrontmatter" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">parseFrontmatter(source: string): { content: string; frontmatter: Record&lt;string, unknown&gt;; }</code><span class="ox-api-entry__description">Parses YAML frontmatter from Markdown content. Uses proper YAML parser for full…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Parses YAML frontmatter from Markdown content. Uses proper YAML parser for full nested object support.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L482-L509">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">source</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">{ ... }</code>
  
</div>
</div>
  </div>
</details>

<details id="ssgtransformoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgTransformOptions</code><span class="ox-api-entry__description">SSG-specific transform options.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>SSG-specific transform options.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L388-L395">View source</a></p>
  </div>
</details>
