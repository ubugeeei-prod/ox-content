# shiki-theme.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts)**

> 5 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>5</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>variables</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="css_variables_theme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const CSS_VARIABLES_THEME = &quot;css-variables&quot;</code><span class="ox-api-entry__description">Name callers pass as highlightTheme to render syntax colors as CSS custom prope…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Name callers pass as <code>highlightTheme</code> to render syntax colors as CSS custom properties instead of baked-in hex values.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export const CSS_VARIABLES_THEME = &quot;css-variables&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts#L8" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="cssvariablestheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">cssVariablesTheme(): ThemeRegistration</code><span class="ox-api-entry__description">Shiki theme whose every color is a --octc-shiki-* custom property. This is what…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns ThemeRegistration</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Shiki theme whose every color is a <code>--octc-shiki-*</code> custom property.</p>
<p>This is what lets syntax highlighting track the active color scheme in both light and dark from a single build: the HTML is generated once, and the properties resolve per mode. A fixed theme like <code>github-dark</code> cannot do that, and lands dark token colors on a light code block.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function cssVariablesTheme(): ThemeRegistration</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts#L43-L51" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">ThemeRegistration</code>

</div>
</div>
  </div>
</details>

<details id="resolvehighlighttheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveHighlightTheme(theme: string | ThemeRegistration): string | ThemeRegistration</code><span class="ox-api-entry__description">Resolves the css-variables alias; any other value passes through.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string | ThemeRegistration</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves the <code>css-variables</code> alias; any other value passes through.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveHighlightTheme(theme: string | ThemeRegistration): string | ThemeRegistration</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts#L54-L58" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">theme</code>
    <code class="ox-api-entry__param-type">string | ThemeRegistration</code>
  </div>

</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string | ThemeRegistration</code>

</div>
</div>
  </div>
</details>

<details id="variable_defaults" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const VARIABLE_DEFAULTS: Record&lt;string, string&gt;</code><span class="ox-api-entry__description">Fallbacks baked into each var() so a site with no color scheme installed still…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Fallbacks baked into each <code>var()</code> so a site with no color scheme installed still renders GitHub Dark colors — the previous default — rather than unstyled text. A <code>@ox-content/theme-color-*</code> package overrides them by defining the same properties per mode.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">const VARIABLE_DEFAULTS: Record&lt;string, string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts#L19-L31" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="variable_prefix" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const VARIABLE_PREFIX = &quot;--octc-shiki-&quot;</code><span class="ox-api-entry__description">Prefix for the emitted properties, matching the rest of the design tokens.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Prefix for the emitted properties, matching the rest of the design tokens.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">const VARIABLE_PREFIX = &quot;--octc-shiki-&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/shiki-theme.ts#L11" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>
