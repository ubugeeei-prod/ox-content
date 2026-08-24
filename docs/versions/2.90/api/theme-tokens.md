# theme-tokens.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-tokens.ts)**

> 2 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>2</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="themetokens" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeTokens = Record&lt;string, string&gt;</code><span class="ox-api-entry__description">Free-form --octc-* custom properties for themes that need more than the typed c…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Free-form <code>--octc-*</code> custom properties for themes that need more than the typed <code>colors</code> / <code>fonts</code> / <code>layout</code> fields.</p>
<p>Keys are written <strong>without</strong> the <code>--octc-</code> prefix, so <code>&quot;surface-glass&quot;</code> becomes <code>--octc-surface-glass</code>. This is the seam that keeps the two theme axes independent: a color package can restyle code-block line markers, brand accents, and surface textures purely through tokens, while a skin package lays out geometry against those same tokens without knowing any color.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type ThemeTokens = Record&lt;string, string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-tokens.ts#L11" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="tokenstocss" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">tokensToCss(light: ThemeTokens, dark: ThemeTokens): string</code><span class="ox-api-entry__description">Renders light and dark token records as the three selectors the SSG runtime swi…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders light and dark token records as the three selectors the SSG runtime switches between: an explicit <code>[data-theme=&quot;dark&quot;]</code> opt-in, the OS <code>prefers-color-scheme</code> fallback, and the <code>:root</code> base.</p>
<p>Emitted after the typed color variables and before the theme&#39;s own <code>css</code>, so a token can override a typed color and raw <code>css</code> can override a token.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function tokensToCss(light: ThemeTokens, dark: ThemeTokens): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-tokens.ts#L24-L40" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">light</code>
    <code class="ox-api-entry__param-type"><a href="#themetokens">ThemeTokens</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">dark</code>
    <code class="ox-api-entry__param-type"><a href="#themetokens">ThemeTokens</a></code>
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
