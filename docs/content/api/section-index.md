# section-index.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts)**

> 6 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>6</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>12</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>10</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="appendsectionindexpages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">appendSectionIndexPages(input: { generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; collectedPages: readonly SectionIndexSourcePage[]; listedPages: readonly SectionIndexSourcePage[]; options?: ResolvedSectionIndexOptions; outDir: string; base: string; extension: string; errors: string[]; render: (page: SectionIndexGeneratedPage) =&gt; Promise&lt;string&gt;; }): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Appends generated section indexes for directories that have no real index.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">10 params</span><span class="ox-api-badge">returns Promise&lt;void&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Appends generated section indexes for directories that have no real index.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function appendSectionIndexPages(input: {
  generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;;
  collectedPages: readonly SectionIndexSourcePage[];
  listedPages: readonly SectionIndexSourcePage[];
  options?: ResolvedSectionIndexOptions;
  outDir: string;
  base: string;
  extension: string;
  errors: string[];
  render: (page: SectionIndexGeneratedPage) =&gt; Promise&lt;string&gt;;
}): Promise&lt;void&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L111-L153" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">{ generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; collectedPages: readonly <a href="#sectionindexsourcepage">SectionIndexSourcePage</a>[]; listedPages: readonly <a href="#sectionindexsourcepage">SectionIndexSourcePage</a>[]; options?: <a href="./types.md#resolvedsectionindexoptions">ResolvedSectionIndexOptions</a>; outDir: string; base: string; extension: string; errors: string[]; render: (page: <a href="#sectionindexgeneratedpage">SectionIndexGeneratedPage</a>) =&gt; Promise&lt;string&gt; }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.generatedPages</code>
    <code class="ox-api-entry__param-type">Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.collectedPages</code>
    <code class="ox-api-entry__param-type">readonly <a href="#sectionindexsourcepage">SectionIndexSourcePage</a>[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.listedPages</code>
    <code class="ox-api-entry__param-type">readonly <a href="#sectionindexsourcepage">SectionIndexSourcePage</a>[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.options?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedsectionindexoptions">ResolvedSectionIndexOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.outDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.base</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.extension</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.errors</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.render</code>
    <code class="ox-api-entry__param-type">(page: <a href="#sectionindexgeneratedpage">SectionIndexGeneratedPage</a>) =&gt; Promise&lt;string&gt;</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;void&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="resolvesectionindexoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveSectionIndexOptions(value: boolean | SectionIndexOptions | undefined): ResolvedSectionIndexOptions</code><span class="ox-api-entry__description">Resolves ssg.sectionIndex with defaults. false / omitted stays off. true enable…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedSectionIndexOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves <code>ssg.sectionIndex</code> with defaults.</p>
<p><code>false</code> / omitted stays off. <code>true</code> enables card listings. An object enables the feature and overrides only the fields the site set.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveSectionIndexOptions(value: boolean | SectionIndexOptions | undefined): ResolvedSectionIndexOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L64-L77" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">boolean | <a href="./types.md#sectionindexoptions">SectionIndexOptions</a> | undefined</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./types.md#resolvedsectionindexoptions">ResolvedSectionIndexOptions</a></code>
  
</div>
</div>
  </div>
</details>

<details id="section-index" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">section-index</code><span class="ox-api-entry__description">Opt-in generated section index pages. Resolution and directory walking live here. Listing HTML is rendered in Rust (ox_…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in generated section index pages.</p>
<p>Resolution and directory walking live here. Listing HTML is rendered in Rust (<code>ox_content_ssg::render_section_index</code>) when the NAPI helper is available; a matching TypeScript renderer covers the same escape / href rules so the SSG path stays safe either way. The Vite plugin appends themed HTML during SSG and never overwrites an existing index page.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L1-L9" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="sectionindexgeneratedpage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SectionIndexGeneratedPage</code><span class="ox-api-entry__description">Synthetic page passed back to generateHtmlPage.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Synthetic page passed back to <code>generateHtmlPage</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SectionIndexGeneratedPage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L50-L56" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="sectionindexgeneratedpage-content">
  <td><code>content</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexgeneratedpage-href">
  <td><code>href</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexgeneratedpage-outputpath">
  <td><code>outputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexgeneratedpage-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexgeneratedpage-urlpath">
  <td><code>urlPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="sectionindexsourcepage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SectionIndexSourcePage</code><span class="ox-api-entry__description">One built page considered when deciding indexes and children.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>One built page considered when deciding indexes and children.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SectionIndexSourcePage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L37-L47" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="sectionindexsourcepage-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexsourcepage-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="sectionindexsourcepage-inputpath">
  <td><code>inputPath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sectionindexsourcepage-routepaths">
  <td><code>routePaths</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ href: string; urlPath: string; outputPath?: string }</code></td>
  <td></td>
</tr>
<tr id="sectionindexsourcepage-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="tosectionindexprocessresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">toSectionIndexProcessResult(page: SectionIndexGeneratedPage): { inputPath: string; routePaths: { outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string; }; transformedHtml: string; title: string; frontmatter: Record&lt;string, unknown&gt;; toc: []; }</code><span class="ox-api-entry__description">Maps a generated section index onto the SSG render shape.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns object</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Maps a generated section index onto the SSG render shape.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function toSectionIndexProcessResult(page: SectionIndexGeneratedPage): {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  transformedHtml: string;
  title: string;
  frontmatter: Record&lt;string, unknown&gt;;
  toc: [];
}</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/section-index.ts#L80-L108" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">page</code>
    <code class="ox-api-entry__param-type"><a href="#sectionindexgeneratedpage">SectionIndexGeneratedPage</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">object</code>
  <div class="ox-api-entry__return-members">
<div class="ox-api-entry__return-member">
<h5>frontmatter</h5>
<code class="ox-api-entry__return-member-type language-typescript">frontmatter: Record&lt;string, unknown&gt;;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>inputPath</h5>
<code class="ox-api-entry__return-member-type language-typescript">inputPath: string;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>routePaths</h5>
<code class="ox-api-entry__return-member-type language-typescript">routePaths: { outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string };</code>
</div>
<div class="ox-api-entry__return-member">
<h5>title</h5>
<code class="ox-api-entry__return-member-type language-typescript">title: string;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>toc</h5>
<code class="ox-api-entry__return-member-type language-typescript">toc: [];</code>
</div>
<div class="ox-api-entry__return-member">
<h5>transformedHtml</h5>
<code class="ox-api-entry__return-member-type language-typescript">transformedHtml: string;</code>
</div>
</div>
</div>
</div>
  </div>
</details>

