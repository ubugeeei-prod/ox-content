# theme.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts)**

> 31 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>31</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>20</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>variables</span>
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
  <strong>118</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="deepmerge" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">deepMerge&lt;T extends Record&lt;string, unknown&gt;&gt;(target: T, source: Partial&lt;T&gt;): T</code><span class="ox-api-entry__description">Deep merge two objects.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns T</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Deep merge two objects.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function deepMerge&lt;T extends Record&lt;string, unknown&gt;&gt;(target: T, source: Partial&lt;T&gt;): T</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L280-L306" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">target</code>
    <code class="ox-api-entry__param-type">T</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">source</code>
    <code class="ox-api-entry__param-type">Partial&lt;T&gt;</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">T</code>
  
</div>
</div>
  </div>
</details>

<details id="defaulttheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const defaultTheme: ThemeConfig</code><span class="ox-api-entry__description">Default theme configuration. Based on the current ox-content SSG styles.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Default theme configuration. Based on the current ox-content SSG styles.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export const defaultTheme: ThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L219-L275" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="definetheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">defineTheme(config: ThemeConfig): ThemeConfig</code><span class="ox-api-entry__description">Defines a theme configuration with type checking.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ThemeConfig</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Defines a theme configuration with type checking.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function defineTheme(config: ThemeConfig): ThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L324-L326" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type"><a href="#themeconfig">ThemeConfig</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#themeconfig">ThemeConfig</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">const myTheme = defineTheme({
  extends: defaultTheme,
  colors: {
    primary: &#39;#3498db&#39;,
  },
  footer: {
    copyright: &#39;2025 My Company&#39;,
  },
});</code></pre>
</div>
</div>
  </div>
</details>

<details id="expandextendschain" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">expandExtendsChain(config: ThemeConfig): ThemeConfig[]</code><span class="ox-api-entry__description">Flattens one layer&#39;s extends chain into base-first order. The seen guard keeps…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ThemeConfig[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Flattens one layer&#39;s <code>extends</code> chain into base-first order.</p>
<p>The <code>seen</code> guard keeps a theme that accidentally extends itself (or forms a cycle through two packages) from hanging the build.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function expandExtendsChain(config: ThemeConfig): ThemeConfig[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L431-L443" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type"><a href="#themeconfig">ThemeConfig</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#themeconfig">ThemeConfig</a>[]</code>
  
</div>
</div>
  </div>
</details>

<details id="legacysociallinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">LegacySocialLinks</code><span class="ox-api-entry__description">Legacy social links configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Legacy social links configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface LegacySocialLinks</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L106-L113" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="legacysociallinks-discord">
  <td><code>discord</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Discord URL</div></td>
</tr>
<tr id="legacysociallinks-github">
  <td><code>github</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">GitHub URL</div></td>
</tr>
<tr id="legacysociallinks-twitter">
  <td><code>twitter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Twitter/X URL</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="mergethemes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">mergeThemes(...themes: (ThemeConfig | ThemeConfig[])[]): ThemeConfig</code><span class="ox-api-entry__description">Merges multiple theme configurations. Later themes override earlier ones. Objec…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ThemeConfig</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Merges multiple theme configurations. Later themes override earlier ones.</p>
<p>Object fields (<code>colors</code>, <code>tokens</code>, <code>layout</code>, …) merge key-by-key, but <code>css</code> and <code>js</code> <strong>concatenate</strong> in layer order — overwriting them would throw away one half of a <code>[skin, colorScheme]</code> stack. Identical fragments are joined once, so a layer reached through both an array and an <code>extends</code> chain does not emit its stylesheet twice.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function mergeThemes(...themes: (ThemeConfig | ThemeConfig[])[]): ThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L343-L369" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">themes</code>
    <code class="ox-api-entry__param-type">(<a href="#themeconfig">ThemeConfig</a> | <a href="#themeconfig">ThemeConfig</a>[])[]</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#themeconfig">ThemeConfig</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">const merged = mergeThemes(defaultTheme, pixelSkin, tokyoNight, overrides);</code></pre>
</div>
</div>
  </div>
</details>

<details id="napisociallinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiSocialLinks</code><span class="ox-api-entry__description">NAPI-compatible social links type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible social links type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiSocialLinks</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L628-L633" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napisociallinks-discord">
  <td><code>discord</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napisociallinks-github">
  <td><code>github</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napisociallinks-links">
  <td><code>links</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">NapiSocialLink[]</code></td>
  <td></td>
</tr>
<tr id="napisociallinks-twitter">
  <td><code>twitter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemecolors" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeColors</code><span class="ox-api-entry__description">NAPI-compatible theme colors type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">10 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme colors type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeColors</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L568-L579" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemecolors-background">
  <td><code>background</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-backgroundalt">
  <td><code>backgroundAlt</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-border">
  <td><code>border</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-codebackground">
  <td><code>codeBackground</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-codebackgroundtop">
  <td><code>codeBackgroundTop</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-codetext">
  <td><code>codeText</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-primary">
  <td><code>primary</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-primaryhover">
  <td><code>primaryHover</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-text">
  <td><code>text</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemecolors-textmuted">
  <td><code>textMuted</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemeconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeConfig</code><span class="ox-api-entry__description">NAPI-compatible theme configuration type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">11 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme configuration type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L660-L672" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemeconfig-colors">
  <td><code>colors</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemecolors">NapiThemeColors</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-css">
  <td><code>css</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-darkcolors">
  <td><code>darkColors</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemecolors">NapiThemeColors</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-embed">
  <td><code>embed</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemeembed">NapiThemeEmbed</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-entrypage">
  <td><code>entryPage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemeentrypage">NapiThemeEntryPage</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-fonts">
  <td><code>fonts</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemefonts">NapiThemeFonts</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-footer">
  <td><code>footer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemefooter">NapiThemeFooter</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-header">
  <td><code>header</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemeheader">NapiThemeHeader</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-js">
  <td><code>js</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-layout">
  <td><code>layout</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napithemelayout">NapiThemeLayout</a></code></td>
  <td></td>
</tr>
<tr id="napithemeconfig-sociallinks">
  <td><code>socialLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#napisociallinks">NapiSocialLinks</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemeembed" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeEmbed</code><span class="ox-api-entry__description">NAPI-compatible theme embed type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme embed type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeEmbed</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L645-L655" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemeembed-contentafter">
  <td><code>contentAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-contentbefore">
  <td><code>contentBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-footer">
  <td><code>footer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-footerbefore">
  <td><code>footerBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-head">
  <td><code>head</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-headerafter">
  <td><code>headerAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-headerbefore">
  <td><code>headerBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-sidebarafter">
  <td><code>sidebarAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeembed-sidebarbefore">
  <td><code>sidebarBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemeentrypage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeEntryPage</code><span class="ox-api-entry__description">NAPI-compatible entry page theme type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible entry page theme type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeEntryPage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L592-L594" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemeentrypage-mode">
  <td><code>mode</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;default&quot; | &quot;subtle&quot;</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemefonts" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeFonts</code><span class="ox-api-entry__description">NAPI-compatible theme fonts type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme fonts type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeFonts</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L584-L587" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemefonts-mono">
  <td><code>mono</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemefonts-sans">
  <td><code>sans</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemefooter" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeFooter</code><span class="ox-api-entry__description">NAPI-compatible theme footer type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme footer type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeFooter</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L620-L623" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemefooter-copyright">
  <td><code>copyright</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemefooter-message">
  <td><code>message</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemeheader" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeHeader</code><span class="ox-api-entry__description">NAPI-compatible theme header type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme header type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeHeader</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L608-L615" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemeheader-logo">
  <td><code>logo</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeheader-logodark">
  <td><code>logoDark</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeheader-logoheight">
  <td><code>logoHeight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="napithemeheader-logolight">
  <td><code>logoLight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemeheader-logowidth">
  <td><code>logoWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="napithemeheader-showsitenametext">
  <td><code>showSiteNameText</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="napithemelayout" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NapiThemeLayout</code><span class="ox-api-entry__description">NAPI-compatible theme layout type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>NAPI-compatible theme layout type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NapiThemeLayout</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L599-L603" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="napithemelayout-headerheight">
  <td><code>headerHeight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemelayout-maxcontentwidth">
  <td><code>maxContentWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="napithemelayout-sidebarwidth">
  <td><code>sidebarWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedthemeconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedThemeConfig</code><span class="ox-api-entry__description">Resolved theme configuration (after merging with defaults).</span><span class="ox-api-entry__meta"><span class="ox-api-badge">15 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved theme configuration (after merging with defaults).</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L197-L213" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedthemeconfig-colors">
  <td><code>colors</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themecolors">ThemeColors</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-css">
  <td><code>css</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-darkcolors">
  <td><code>darkColors</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themecolors">ThemeColors</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-darktokens">
  <td><code>darkTokens</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-tokens.md#themetokens">ThemeTokens</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-embed">
  <td><code>embed</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeembed">ThemeEmbed</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-entrypage">
  <td><code>entryPage</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeentrypage">ThemeEntryPage</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-fonts">
  <td><code>fonts</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themefonts">ThemeFonts</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-footer">
  <td><code>footer</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themefooter">ThemeFooter</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-header">
  <td><code>header</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeheader">ThemeHeader</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-js">
  <td><code>js</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-layout">
  <td><code>layout</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themelayout">ThemeLayout</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-sidebar">
  <td><code>sidebar</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">SidebarItem[]</code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-sociallinks">
  <td><code>socialLinks</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#sociallinks">SocialLinks</a></code></td>
  <td></td>
</tr>
<tr id="resolvedthemeconfig-tokens">
  <td><code>tokens</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-tokens.md#themetokens">ThemeTokens</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvetheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveTheme(config?: ThemeConfig | ThemeConfig[]): ResolvedThemeConfig</code><span class="ox-api-entry__description">Resolves a theme configuration by merging with its extends chain and defaults.…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedThemeConfig</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves a theme configuration by merging with its extends chain and defaults.</p>
<p>An array composes independent layers left to right, which is how a skin package and a color package are stacked:</p>
<pre><code class="language-ts">resolveTheme([pixelSkin, tokyoNight, { footer: { copyright: &quot;2026&quot; } }]);</code></pre>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveTheme(config?: ThemeConfig | ThemeConfig[]): ResolvedThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L390-L423" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type"><a href="#themeconfig">ThemeConfig</a> | <a href="#themeconfig">ThemeConfig</a>[]</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#resolvedthemeconfig">ResolvedThemeConfig</a></code>
  
</div>
</div>
  </div>
</details>

<details id="sociallink" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SocialLink</code><span class="ox-api-entry__description">Custom social link.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Custom social link.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SocialLink</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L99-L103" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="sociallink-arialabel">
  <td><code>ariaLabel</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="sociallink-icon">
  <td><code>icon</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#sociallinkicon">SocialLinkIcon</a></code></td>
  <td></td>
</tr>
<tr id="sociallink-link">
  <td><code>link</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="sociallinkicon" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SocialLinkIcon = string | { svg: string }</code><span class="ox-api-entry__description">Custom social link icon.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Custom social link icon.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type SocialLinkIcon = string | { svg: string }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L96" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="sociallinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SocialLinks = LegacySocialLinks | SocialLink[]</code><span class="ox-api-entry__description">Social links configuration.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Social links configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type SocialLinks = LegacySocialLinks | SocialLink[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L116" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="theme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">theme</code><span class="ox-api-entry__description">Theme API for ox-content SSG Provides VitePress-like theming with default theme + customization.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme API for ox-content SSG</p>
<p>Provides VitePress-like theming with default theme + customization.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L1-L5" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="themecolors" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeColors</code><span class="ox-api-entry__description">Theme color configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">10 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme color configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeColors</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L14-L35" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themecolors-background">
  <td><code>background</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Background color</div></td>
</tr>
<tr id="themecolors-backgroundalt">
  <td><code>backgroundAlt</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Alternative background color (sidebar, code blocks)</div></td>
</tr>
<tr id="themecolors-border">
  <td><code>border</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Border color</div></td>
</tr>
<tr id="themecolors-codebackground">
  <td><code>codeBackground</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Code block background color</div></td>
</tr>
<tr id="themecolors-codebackgroundtop">
  <td><code>codeBackgroundTop</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Code block gradient color at the top; defaults to <code>codeBackground</code> when customized</div></td>
</tr>
<tr id="themecolors-codetext">
  <td><code>codeText</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Code block text color</div></td>
</tr>
<tr id="themecolors-primary">
  <td><code>primary</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Primary accent color</div></td>
</tr>
<tr id="themecolors-primaryhover">
  <td><code>primaryHover</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Primary color on hover</div></td>
</tr>
<tr id="themecolors-text">
  <td><code>text</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Main text color</div></td>
</tr>
<tr id="themecolors-textmuted">
  <td><code>textMuted</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Muted/secondary text color</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themeconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeConfig</code><span class="ox-api-entry__description">Complete theme configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">16 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Complete theme configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L153-L192" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themeconfig-colors">
  <td><code>colors</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themecolors">ThemeColors</a></code></td>
  <td><div class="ox-api-entry__member-description">Light mode colors (maps to CSS variables)</div></td>
</tr>
<tr id="themeconfig-css">
  <td><code>css</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Additional custom CSS. Composed layers <strong>concatenate</strong> this rather than<br>overwrite, so stacking a skin and a color scheme keeps both stylesheets.</div></td>
</tr>
<tr id="themeconfig-darkcolors">
  <td><code>darkColors</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themecolors">ThemeColors</a></code></td>
  <td><div class="ox-api-entry__member-description">Dark mode colors (maps to CSS variables)</div></td>
</tr>
<tr id="themeconfig-darktokens">
  <td><code>darkTokens</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-tokens.md#themetokens">ThemeTokens</a></code></td>
  <td><div class="ox-api-entry__member-description">Extra <code>--octc-*</code> custom properties for dark mode.</div></td>
</tr>
<tr id="themeconfig-embed">
  <td><code>embed</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeembed">ThemeEmbed</a></code></td>
  <td><div class="ox-api-entry__member-description">Embedded HTML content at specific positions</div></td>
</tr>
<tr id="themeconfig-entrypage">
  <td><code>entryPage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeentrypage">ThemeEntryPage</a></code></td>
  <td><div class="ox-api-entry__member-description">Entry page configuration</div></td>
</tr>
<tr id="themeconfig-extends">
  <td><code>extends</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeconfig">ThemeConfig</a></code></td>
  <td><div class="ox-api-entry__member-description">Base theme to extend</div></td>
</tr>
<tr id="themeconfig-fonts">
  <td><code>fonts</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themefonts">ThemeFonts</a></code></td>
  <td><div class="ox-api-entry__member-description">Font configuration (maps to CSS variables)</div></td>
</tr>
<tr id="themeconfig-footer">
  <td><code>footer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themefooter">ThemeFooter</a></code></td>
  <td><div class="ox-api-entry__member-description">Footer configuration</div></td>
</tr>
<tr id="themeconfig-header">
  <td><code>header</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themeheader">ThemeHeader</a></code></td>
  <td><div class="ox-api-entry__member-description">Header configuration</div></td>
</tr>
<tr id="themeconfig-js">
  <td><code>js</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Additional custom JavaScript. Concatenated across composed layers.</div></td>
</tr>
<tr id="themeconfig-layout">
  <td><code>layout</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themelayout">ThemeLayout</a></code></td>
  <td><div class="ox-api-entry__member-description">Layout configuration (maps to CSS variables)</div></td>
</tr>
<tr id="themeconfig-name">
  <td><code>name</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Theme name for identification</div></td>
</tr>
<tr id="themeconfig-sidebar">
  <td><code>sidebar</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">SidebarItem[]</code></td>
  <td></td>
</tr>
<tr id="themeconfig-sociallinks">
  <td><code>socialLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#sociallinks">SocialLinks</a></code></td>
  <td><div class="ox-api-entry__member-description">Social links configuration</div></td>
</tr>
<tr id="themeconfig-tokens">
  <td><code>tokens</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-tokens.md#themetokens">ThemeTokens</a></code></td>
  <td><div class="ox-api-entry__member-description">Extra <code>--octc-*</code> custom properties for light mode, keyed without the<br>prefix. Merged key-by-key across composed layers, so a later layer can<br>restyle one token without redeclaring the rest.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themecss" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">themeCss(theme: ResolvedThemeConfig): string</code><span class="ox-api-entry__description">Token blocks come first so a theme&#39;s own css stays the final word, and both lan…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Token blocks come first so a theme&#39;s own <code>css</code> stays the final word, and both land after the typed color variables the Rust renderer emits.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function themeCss(theme: ResolvedThemeConfig): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L542-L548" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">theme</code>
    <code class="ox-api-entry__param-type"><a href="#resolvedthemeconfig">ResolvedThemeConfig</a></code>
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

<details id="themeembed" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeEmbed</code><span class="ox-api-entry__description">Embedded HTML content for specific positions in the page layout.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Embedded HTML content for specific positions in the page layout.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeEmbed</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L121-L140" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themeembed-contentafter">
  <td><code>contentAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content after main content</div></td>
</tr>
<tr id="themeembed-contentbefore">
  <td><code>contentBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content before main content</div></td>
</tr>
<tr id="themeembed-footer">
  <td><code>footer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Custom footer content (replaces default footer)</div></td>
</tr>
<tr id="themeembed-footerbefore">
  <td><code>footerBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content before footer</div></td>
</tr>
<tr id="themeembed-head">
  <td><code>head</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content to embed into &lt;head&gt;</div></td>
</tr>
<tr id="themeembed-headerafter">
  <td><code>headerAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content after header</div></td>
</tr>
<tr id="themeembed-headerbefore">
  <td><code>headerBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content before header</div></td>
</tr>
<tr id="themeembed-sidebarafter">
  <td><code>sidebarAfter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content after sidebar navigation</div></td>
</tr>
<tr id="themeembed-sidebarbefore">
  <td><code>sidebarBefore</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Content before sidebar navigation</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themeentrypage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeEntryPage</code><span class="ox-api-entry__description">Entry page theme configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Entry page theme configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeEntryPage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L62-L65" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themeentrypage-mode">
  <td><code>mode</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;default&quot; | &quot;subtle&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Landing page presentation mode</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themefonts" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeFonts</code><span class="ox-api-entry__description">Theme font configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme font configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeFonts</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L52-L57" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themefonts-mono">
  <td><code>mono</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Monospace font stack</div></td>
</tr>
<tr id="themefonts-sans">
  <td><code>sans</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Sans-serif font stack</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themefooter" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeFooter</code><span class="ox-api-entry__description">Theme footer configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme footer configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeFooter</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L88-L93" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themefooter-copyright">
  <td><code>copyright</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Copyright text (supports HTML)</div></td>
</tr>
<tr id="themefooter-message">
  <td><code>message</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Footer message (supports HTML)</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themeheader" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeHeader</code><span class="ox-api-entry__description">Theme header configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme header configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeHeader</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L70-L83" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themeheader-logo">
  <td><code>logo</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Logo image URL</div></td>
</tr>
<tr id="themeheader-logodark">
  <td><code>logoDark</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Dark mode logo image URL</div></td>
</tr>
<tr id="themeheader-logoheight">
  <td><code>logoHeight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Logo height in pixels</div></td>
</tr>
<tr id="themeheader-logolight">
  <td><code>logoLight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Light mode logo image URL</div></td>
</tr>
<tr id="themeheader-logowidth">
  <td><code>logoWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Logo width in pixels</div></td>
</tr>
<tr id="themeheader-showsitenametext">
  <td><code>showSiteNameText</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Whether to render the site name text next to the logo</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themelayout" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeLayout</code><span class="ox-api-entry__description">Theme layout configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme layout configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeLayout</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L40-L47" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themelayout-headerheight">
  <td><code>headerHeight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Header height (CSS value, e.g., &quot;60px&quot;)</div></td>
</tr>
<tr id="themelayout-maxcontentwidth">
  <td><code>maxContentWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Maximum content width (CSS value, e.g., &quot;960px&quot;)</div></td>
</tr>
<tr id="themelayout-sidebarwidth">
  <td><code>sidebarWidth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Sidebar width (CSS value, e.g., &quot;260px&quot;)</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themetonapi" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">themeToNapi(theme: ResolvedThemeConfig): NapiThemeConfig</code><span class="ox-api-entry__description">Converts resolved theme to the format expected by Rust NAPI.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns NapiThemeConfig</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Converts resolved theme to the format expected by Rust NAPI.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function themeToNapi(theme: ResolvedThemeConfig): NapiThemeConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme.ts#L463-L536" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">theme</code>
    <code class="ox-api-entry__param-type"><a href="#resolvedthemeconfig">ResolvedThemeConfig</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#napithemeconfig">NapiThemeConfig</a></code>
  
</div>
</div>
  </div>
</details>

