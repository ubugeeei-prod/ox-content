# transform.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts)**

> 9 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>9</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>41</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="extractimports" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">extractImports(content: string): string[]</code><span class="ox-api-entry__description">Extracts imports from Markdown content. Supports importing components for inter…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extracts imports from Markdown content.</p>
<p>Supports importing components for interactive islands.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function extractImports(content: string): string[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L737-L747" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateModuleCode(html: string, frontmatter: Record&lt;string, unknown&gt;, toc: TocEntry[], filePath: string, _options: ResolvedOptions): string</code><span class="ox-api-entry__description">Generates the JavaScript module code.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates the JavaScript module code.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function generateModuleCode(html: string, frontmatter: Record&lt;string, unknown&gt;, toc: TocEntry[], filePath: string, _options: ResolvedOptions): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L681-L730" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
    <code class="ox-api-entry__param-type">Record&lt;string, unknown&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">toc</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#tocentry">TocEntry</a>[]</code>
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
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedoptions">ResolvedOptions</a></code>
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
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateOgImageSvg(data: OgImageData, config?: OgImageConfig): Promise&lt;string | null&gt;</code><span class="ox-api-entry__description">Generates an OG image SVG using the Rust-based generator. This function uses th…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;string | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates an OG image SVG using the Rust-based generator.</p>
<p>This function uses the Rust NAPI bindings to generate SVG-based OG images for social media previews. The SVG can be served directly or converted to PNG/JPEG for broader compatibility.</p>
<p>In the future, custom JS templates can be provided to override the default Rust-based template.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function generateOgImageSvg(data: OgImageData, config?: OgImageConfig): Promise&lt;string | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L763-L785" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">data</code>
    <code class="ox-api-entry__param-type"><a href="#ogimagedata">OgImageData</a></code>
  </div>
  <p class="ox-api-entry__param-description">OG image data (title, description, etc.)</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type"><a href="#ogimageconfig">OgImageConfig</a></code>
  </div>
  <p class="ox-api-entry__param-description">Optional OG image configuration — optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string | null&gt;</code>
  <p class="ox-api-entry__return-description">SVG string or null if NAPI bindings are unavailable</p>
</div>
</div>
  </div>
</details>

<details id="jstransformoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">JsTransformOptions</code><span class="ox-api-entry__description">Options for Rust-based Markdown transformation. Controls which Markdown extensi…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">22 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for Rust-based Markdown transformation.</p>
<p>Controls which Markdown extensions and features are enabled during parsing and rendering.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">interface JsTransformOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L141-L268" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="jstransformoptions-attributes">
  <td><code>attributes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ enabled?: boolean }</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-autolinks">
  <td><code>autolinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable automatic link conversion (URLs become clickable).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-baseurl">
  <td><code>baseUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base URL for absolute link conversion (e.g., &quot;/&quot; or &quot;/docs/&quot;).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;/&quot;</code></div></td>
</tr>
<tr id="jstransformoptions-cjkemphasis">
  <td><code>cjkEmphasis</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-codeannotationdefaultlinenumbers">
  <td><code>codeAnnotationDefaultLineNumbers</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable line numbers for all code blocks by default.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-codeannotationmetakey">
  <td><code>codeAnnotationMetaKey</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Fence meta key used to read code annotations.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;annotate&quot;</code></div></td>
</tr>
<tr id="jstransformoptions-codeannotations">
  <td><code>codeAnnotations</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable line annotations for code blocks using fence meta.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-codeannotationsyntax">
  <td><code>codeAnnotationSyntax</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;attribute&quot; | &quot;vitepress&quot; | &quot;both&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Code annotation syntax mode.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;attribute&quot;</code></div></td>
</tr>
<tr id="jstransformoptions-codeimports">
  <td><code>codeImports</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ enabled?: boolean; rootDir?: string }</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-convertmdlinks">
  <td><code>convertMdLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Convert <code>.md</code> links to <code>.html</code> links for SSG output.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-editthispage">
  <td><code>editThisPage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ enabled?: boolean; repoUrl?: string; branch?: string; rootDir?: string; label?: string }</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-emojishortcodes">
  <td><code>emojiShortcodes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ enabled?: boolean; custom?: Record&lt;string, string&gt; }</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-footnotes">
  <td><code>footnotes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable footnotes syntax ([^1]: definition).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-frontmatter">
  <td><code>frontmatter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Parse YAML frontmatter before transforming.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="jstransformoptions-gfm">
  <td><code>gfm</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable GitHub Flavored Markdown extensions.<br>Includes tables, task lists, strikethrough, and autolinks.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-sanitize">
  <td><code>sanitize</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">JsSanitizeOptions</code></td>
  <td></td>
</tr>
<tr id="jstransformoptions-sourcepath">
  <td><code>sourcePath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source file path for relative link resolution.<br>Used to determine if the current file is an index file.</div></td>
</tr>
<tr id="jstransformoptions-strikethrough">
  <td><code>strikethrough</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable strikethrough syntax (~~text~~).<br>Requires GFM to be enabled.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-tables">
  <td><code>tables</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable table rendering (GFM extension).<br>Requires GFM to be enabled for full functionality.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-tasklists">
  <td><code>taskLists</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable task list syntax (- [ ] unchecked, - [x] checked).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="jstransformoptions-tocmaxdepth">
  <td><code>tocMaxDepth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Maximum heading depth for table of contents.<br>Headings deeper than this level are excluded from TOC.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">3</code></div></td>
</tr>
<tr id="jstransformoptions-wikilinks">
  <td><code>wikiLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ enabled?: boolean; baseUrl?: string }</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napibindings" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiBindings</code><span class="ox-api-entry__description">NAPI bindings for Rust-based Markdown processing. Provides access to compiled R…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI bindings for Rust-based Markdown processing.</p>
<p>Provides access to compiled Rust functions for high-performance Markdown parsing and rendering operations.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">interface NapiBindings</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L48-L101" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napibindings-generateogimagesvg">
  <td><code>generateOgImageSvg</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(data: <a href="#ogimagedata">OgImageData</a>, config?: <a href="#ogimageconfig">OgImageConfig</a>) =&gt; string</code></td>
  <td><div class="ox-api-entry__member-description">Generates an OG image as SVG.</div><ul class="ox-api-entry__member-params"><li><code>data</code> OG image data (title, description, etc.)</li><li><code>config</code> Optional OG image configuration — optional</li></ul><div class="ox-api-entry__member-return"><span>Returns</span> SVG string</div></td>
</tr>
<tr id="napibindings-lintcodeblocks">
  <td><code>lintCodeBlocks</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(source: string, options?: JsCodeBlockLintOptions) =&gt; JsCodeBlockDiagnostic[]</code></td>
  <td><ul class="ox-api-entry__member-params"><li><code>source</code></li><li><code>options</code> optional</li></ul></td>
</tr>
<tr id="napibindings-mergehighlightedcodeblocks">
  <td><code>mergeHighlightedCodeBlocks</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(originalHtml: string, highlightedHtml: string) =&gt; string</code></td>
  <td><div class="ox-api-entry__member-description">Restores code block metadata after JavaScript-side syntax highlighting.</div><ul class="ox-api-entry__member-params"><li><code>originalHtml</code> HTML before syntax highlighting</li><li><code>highlightedHtml</code> HTML after Shiki highlighting</li></ul><div class="ox-api-entry__member-return"><span>Returns</span> Highlighted HTML with original code block metadata reapplied</div></td>
</tr>
<tr id="napibindings-parseandrender">
  <td><code>parseAndRender</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(source: string, options?: { gfm?: boolean }) =&gt; { html: string; errors: string[] }</code></td>
  <td><div class="ox-api-entry__member-description">Simple Markdown parser and renderer in one step.<br>Faster for simple use cases but lacks advanced features.</div><ul class="ox-api-entry__member-params"><li><code>source</code> Raw Markdown content</li><li><code>options</code> Parser configuration (GFM flag) — optional</li><li><code>options.gfm?</code> optional</li></ul><div class="ox-api-entry__member-return"><span>Returns</span> Rendered HTML and parsing errors</div></td>
</tr>
<tr id="napibindings-sanitizehtml">
  <td><code>sanitizeHtml</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(html: string, options?: JsSanitizeOptions) =&gt; string</code></td>
  <td><ul class="ox-api-entry__member-params"><li><code>html</code></li><li><code>options</code> optional</li></ul></td>
</tr>
<tr id="napibindings-transform">
  <td><code>transform</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(source: string, options?: <a href="#jstransformoptions">JsTransformOptions</a>) =&gt; { html: string; frontmatter: string; toc: Array&lt;{ depth: number; text: string; slug: string; children?: <a href="./types.md#tocentry">TocEntry</a>[] }&gt;; errors: string[] }</code></td>
  <td><div class="ox-api-entry__member-description">Full-featured Markdown transformation pipeline.<br>Handles frontmatter extraction, TOC generation, and advanced parsing.</div><ul class="ox-api-entry__member-params"><li><code>source</code> Raw Markdown content (may include frontmatter)</li><li><code>options</code> Comprehensive transformation options — optional</li></ul><div class="ox-api-entry__member-return"><span>Returns</span> Transformed result with HTML, metadata, and TOC</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ogimageconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageConfig</code><span class="ox-api-entry__description">OG image configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L120-L133" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimageconfig-backgroundcolor">
  <td><code>backgroundColor</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Background color (hex)</div></td>
</tr>
<tr id="ogimageconfig-descriptionfontsize">
  <td><code>descriptionFontSize</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Description font size</div></td>
</tr>
<tr id="ogimageconfig-height">
  <td><code>height</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image height in pixels</div></td>
</tr>
<tr id="ogimageconfig-textcolor">
  <td><code>textColor</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Text color (hex)</div></td>
</tr>
<tr id="ogimageconfig-titlefontsize">
  <td><code>titleFontSize</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Title font size</div></td>
</tr>
<tr id="ogimageconfig-width">
  <td><code>width</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image width in pixels</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ogimagedata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageData</code><span class="ox-api-entry__description">OG image data for generating social media preview images.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image data for generating social media preview images.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageData</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L106-L115" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimagedata-author">
  <td><code>author</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Author name</div></td>
</tr>
<tr id="ogimagedata-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page description</div></td>
</tr>
<tr id="ogimagedata-sitename">
  <td><code>siteName</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site name</div></td>
</tr>
<tr id="ogimagedata-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page title</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ssgtransformoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgTransformOptions</code><span class="ox-api-entry__description">SSG-specific transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>SSG-specific transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgTransformOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L455-L462" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgtransformoptions-baseurl">
  <td><code>baseUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base URL for absolute link conversion</div></td>
</tr>
<tr id="ssgtransformoptions-convertmdlinks">
  <td><code>convertMdLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Convert <code>.md</code> links to <code>.html</code> links</div></td>
</tr>
<tr id="ssgtransformoptions-sourcepath">
  <td><code>sourcePath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source file path for relative link resolution</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="transform" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">transform</code><span class="ox-api-entry__description">Markdown Transformation Engine This module handles the complete transformation pipeline for Markdown files, converting…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Markdown Transformation Engine</p>
<p>This module handles the complete transformation pipeline for Markdown files, converting raw Markdown content into JavaScript modules that can be imported by web applications. The transformation process includes:</p>
<ol>
<li><strong>Parsing</strong>: Uses Rust-based parser via NAPI bindings for high performance</li>
<li><strong>Rendering</strong>: Converts parsed AST to semantic HTML</li>
<li><strong>Enhancement</strong>: Applies syntax highlighting, Mermaid diagram rendering, etc.</li>
<li><strong>Code Generation</strong>: Generates JavaScript/TypeScript module code</li>
</ol>
<p>The generated modules export:</p>
<ul>
<li><code>html</code>: Rendered HTML content</li>
<li><code>frontmatter</code>: Parsed YAML metadata</li>
<li><code>toc</code>: Hierarchical table of contents</li>
<li><code>render</code>: Client-side render function for dynamic updates</li>
</ul>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/transform.ts#L1-L32" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-typescript">import { transformMarkdown } from &#39;./transform&#39;;

const content = await transformMarkdown(
&#39;# Hello\n\nWorld&#39;,
&#39;path/to/file.md&#39;,
resolvedOptions
);

console.log(content.html); // &#39;&lt;h1&gt;Hello&lt;/h1&gt;&lt;p&gt;World&lt;/p&gt;&#39;
console.log(content.toc); // [{ depth: 1, text: &#39;Hello&#39;, slug: &#39;hello&#39;, children: [] }]</code></pre>

</div>
</div>
  </div>
</details>
