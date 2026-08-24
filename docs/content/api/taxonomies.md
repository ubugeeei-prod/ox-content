# taxonomies.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts)**

> 7 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>7</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>14</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="appendtaxonomypages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">appendTaxonomyPages(input: { generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; listedPages: readonly TaxonomySourcePage[]; options?: ResolvedTaxonomiesOptions; outDir: string; base: string; render: (page: TaxonomyGeneratedPage) =&gt; Promise&lt;string&gt;; errors: string[]; }): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Renders themed list and per-term pages and appends them to the build.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">8 params</span><span class="ox-api-badge">returns Promise&lt;void&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders themed list and per-term pages and appends them to the build.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function appendTaxonomyPages(input: {
  generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;;
  listedPages: readonly TaxonomySourcePage[];
  options?: ResolvedTaxonomiesOptions;
  outDir: string;
  base: string;
  render: (page: TaxonomyGeneratedPage) =&gt; Promise&lt;string&gt;;
  errors: string[];
}): Promise&lt;void&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L153-L182" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">{ generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; listedPages: readonly <a href="./taxonomies-html.md#taxonomysourcepage">TaxonomySourcePage</a>[]; options?: <a href="./types.md#resolvedtaxonomiesoptions">ResolvedTaxonomiesOptions</a>; outDir: string; base: string; render: (page: <a href="#taxonomygeneratedpage">TaxonomyGeneratedPage</a>) =&gt; Promise&lt;string&gt;; errors: string[] }</code>
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
    <code class="ox-api-entry__param-name">input.listedPages</code>
    <code class="ox-api-entry__param-type">readonly <a href="./taxonomies-html.md#taxonomysourcepage">TaxonomySourcePage</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.options?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedtaxonomiesoptions">ResolvedTaxonomiesOptions</a></code>
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
    <code class="ox-api-entry__param-name">input.render</code>
    <code class="ox-api-entry__param-type">(page: <a href="#taxonomygeneratedpage">TaxonomyGeneratedPage</a>) =&gt; Promise&lt;string&gt;</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.errors</code>
    <code class="ox-api-entry__param-type">string[]</code>
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

<details id="injectrelatedpages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">injectRelatedPages(pages: TaxonomySourcePage[], listed: readonly TaxonomySourcePage[], options?: ResolvedTaxonomiesOptions): void</code><span class="ox-api-entry__description">Appends related-page HTML to source pages that share a listed term.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns void</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Appends related-page HTML to source pages that share a listed term.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function injectRelatedPages(pages: TaxonomySourcePage[], listed: readonly TaxonomySourcePage[], options?: ResolvedTaxonomiesOptions): void</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L83-L119" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">pages</code>
    <code class="ox-api-entry__param-type"><a href="./taxonomies-html.md#taxonomysourcepage">TaxonomySourcePage</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">listed</code>
    <code class="ox-api-entry__param-type">readonly <a href="./taxonomies-html.md#taxonomysourcepage">TaxonomySourcePage</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedtaxonomiesoptions">ResolvedTaxonomiesOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">void</code>
</div>
</div>
  </div>
</details>

<details id="resolvetaxonomiesoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveTaxonomiesOptions(value: boolean | TaxonomiesOptions | undefined): ResolvedTaxonomiesOptions</code><span class="ox-api-entry__description">Resolves taxonomies with defaults. false / omitted stays off. true enables tags…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedTaxonomiesOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves <code>taxonomies</code> with defaults.</p>
<p><code>false</code> / omitted stays off. <code>true</code> enables <code>tags</code> and <code>categories</code> with relatedLimit 5. An object enables the feature and overrides only set fields.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveTaxonomiesOptions(value: boolean | TaxonomiesOptions | undefined): ResolvedTaxonomiesOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L41-L63" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">boolean | <a href="./types.md#taxonomiesoptions">TaxonomiesOptions</a> | undefined</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./types.md#resolvedtaxonomiesoptions">ResolvedTaxonomiesOptions</a></code>
</div>
</div>
  </div>
</details>

<details id="taxonomies" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">taxonomies</code><span class="ox-api-entry__description">Opt-in taxonomy term pages and related-page lists. Resolution and HTML live here. The Vite plugin injects related marku…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in taxonomy term pages and related-page lists.</p>
<p>Resolution and HTML live here. The Vite plugin injects related markup into page content, then writes themed list and per-term pages during SSG.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L1-L6" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="taxonomygeneratedpage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TaxonomyGeneratedPage</code><span class="ox-api-entry__description">Synthetic page passed back to generateHtmlPage.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Synthetic page passed back to <code>generateHtmlPage</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TaxonomyGeneratedPage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L27-L33" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="taxonomygeneratedpage-content">
  <td><code>content</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="taxonomygeneratedpage-href">
  <td><code>href</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="taxonomygeneratedpage-outputpath">
  <td><code>outputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="taxonomygeneratedpage-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="taxonomygeneratedpage-urlpath">
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

<details id="termslug" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">termSlug(term: string): string | undefined</code><span class="ox-api-entry__description">Stable URL slug for a frontmatter term. Returns undefined when the value cannot…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string | undefined</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Stable URL slug for a frontmatter term.</p>
<p>Returns <code>undefined</code> when the value cannot become a safe <code>[a-z0-9-]</code> href.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function termSlug(term: string): string | undefined</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L70-L80" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">term</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string | undefined</code>
</div>
</div>
  </div>
</details>

<details id="totaxonomyprocessresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">toTaxonomyProcessResult(page: TaxonomyGeneratedPage): { inputPath: string; routePaths: { outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string; }; transformedHtml: string; title: string; frontmatter: Record&lt;string, unknown&gt;; toc: []; }</code><span class="ox-api-entry__description">Maps a generated taxonomy page onto the SSG render shape.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns object</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Maps a generated taxonomy page onto the SSG render shape.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function toTaxonomyProcessResult(page: TaxonomyGeneratedPage): {
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/taxonomies.ts#L122-L150" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">page</code>
    <code class="ox-api-entry__param-type"><a href="#taxonomygeneratedpage">TaxonomyGeneratedPage</a></code>
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
