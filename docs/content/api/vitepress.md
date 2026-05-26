# vitepress.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts)**

> 5 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>5</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>parameters</span>
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

<details id="convertvitepressnav" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">convertVitePressNav(nav: VitePressNavItem[]): SsgNavigationGroup[]</code><span class="ox-api-entry__description">Converts VitePress top navigation into ox-content sidebar groups. This is used…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns SsgNavigationGroup[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Converts VitePress top navigation into ox-content sidebar groups. This is used as a fallback when no explicit sidebar is defined.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function convertVitePressNav(nav: VitePressNavItem[]): SsgNavigationGroup[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts#L375-L403" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">nav</code>
    <code class="ox-api-entry__param-type">VitePressNavItem[]</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">SsgNavigationGroup[]</code>
  
</div>
</div>
  </div>
</details>

<details id="convertvitepresssidebar" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">convertVitePressSidebar(sidebar: VitePressSidebar): SsgNavigationGroup[]</code><span class="ox-api-entry__description">Converts a VitePress sidebar config into ox-content navigation groups. Nested V…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns SsgNavigationGroup[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Converts a VitePress sidebar config into ox-content navigation groups. Nested VitePress items are flattened into the nearest ox-content group.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function convertVitePressSidebar(sidebar: VitePressSidebar): SsgNavigationGroup[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts#L359-L369" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">sidebar</code>
    <code class="ox-api-entry__param-type">VitePressSidebar</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">SsgNavigationGroup[]</code>
  
</div>
</div>
  </div>
</details>

<details id="fromvitepressconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">fromVitePressConfig(config: VitePressConfig, overrides: OxContentOptions = {}): OxContentOptions</code><span class="ox-api-entry__description">Creates ox-content plugin options from an existing VitePress config.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns OxContentOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates ox-content plugin options from an existing VitePress config.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function fromVitePressConfig(config: VitePressConfig, overrides: OxContentOptions = {}): OxContentOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts#L408-L436" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type">VitePressConfig</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">overrides</code>
    <code class="ox-api-entry__param-type">OxContentOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">OxContentOptions</code>
  
</div>
</div>
  </div>
</details>

<details id="generatevitepressmigrationconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateVitePressMigrationConfig(config: VitePressConfig, overrides: OxContentOptions = {}, options: GenerateVitePressMigrationConfigOptions = {}): string</code><span class="ox-api-entry__description">Generates a TypeScript module exporting migrated ox-content options. This is us…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates a TypeScript module exporting migrated ox-content options.</p>
<p>This is used by the migration CLI so users can inspect and edit the resulting object instead of keeping a runtime dependency on their VitePress config.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function generateVitePressMigrationConfig(config: VitePressConfig, overrides: OxContentOptions = {}, options: GenerateVitePressMigrationConfigOptions = {}): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts#L444-L458" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type">VitePressConfig</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">overrides</code>
    <code class="ox-api-entry__param-type">OxContentOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">GenerateVitePressMigrationConfigOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
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

<details id="normalizevitepressfrontmatter" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">normalizeVitePressFrontmatter(frontmatter: Record&lt;string, unknown&gt;): Record&lt;string, unknown&gt;</code><span class="ox-api-entry__description">Normalizes VitePress-specific frontmatter into ox-content&#39;s entry-page shape.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns Record</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Normalizes VitePress-specific frontmatter into ox-content&#39;s entry-page shape.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function normalizeVitePressFrontmatter(frontmatter: Record&lt;string, unknown&gt;): Record&lt;string, unknown&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/vitepress.ts#L556-L576" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">frontmatter</code>
    <code class="ox-api-entry__param-type">Record</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Record</code>
  
</div>
</div>
  </div>
</details>
