# types.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts)**

> 113 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>113</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>104</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>545</strong>
  <span>members</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="a11yoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">A11yOptions</code><span class="ox-api-entry__description">Per-control flags for ssg.a11y. Omitted fields keep the defaults when the featu…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Per-control flags for <code>ssg.a11y</code>.</p>
<p>Omitted fields keep the defaults when the feature itself is enabled.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface A11yOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L461-L468" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="a11yoptions-skiplinklabel">
  <td><code>skipLinkLabel</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Visible label for the skip link. Escaped in HTML.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;Skip to content&quot;</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="attrsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">AttrsOptions</code><span class="ox-api-entry__description">Options for markdown-it-attrs style attribute blocks. Attribute blocks let auth…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for markdown-it-attrs style attribute blocks.</p>
<p>Attribute blocks let authors attach IDs, classes, and key/value attributes to nearby Markdown nodes with syntax such as <code>{#install .lead}</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface AttrsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1565-L1576" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="attrsoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the attrs transform when an options object is supplied.<br><br>Set to <code>false</code> to keep the object shape while disabling the transform.<br>This is mainly useful for config merging where callers want to preserve a<br>stable object structure.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="badgeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BadgeOptions</code><span class="ox-api-entry__description">Options for opt-in {badge:variant} inline badges.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>{badge:variant}</code> inline badges.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BadgeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1409-L1416" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="badgeoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the badge transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="builtinembedoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BuiltinEmbedOptions</code><span class="ox-api-entry__description">Built-in embed configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">8 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Built-in embed configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BuiltinEmbedOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1322-L1379" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="builtinembedoptions-bluesky">
  <td><code>bluesky</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;Bluesky&gt;</code> as static cards.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="builtinembedoptions-github">
  <td><code>github</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | GitHubOptions</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;GitHub repo=&quot;owner/name&quot; /&gt;</code> repository cards.<br>Pass an options object to configure fetching.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="builtinembedoptions-opengraph">
  <td><code>openGraph</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | OgpOptions</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;OgCard url=&quot;https://example.com&quot; /&gt;</code> Open Graph link cards.<br>Pass an options object to configure fetching.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="builtinembedoptions-pm">
  <td><code>pm</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#builtinpmoptions">BuiltinPmOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Expand <code>&lt;pm&gt;npm install …&lt;/pm&gt;</code> blocks into npm/pnpm/yarn/bun install tabs.<br><br>Accepts a boolean to toggle the feature, or an options object to opt in to<br>synced tab groups. Synced groups are OFF by default; when enabled with<br><code>{ sync: true }</code>, selecting a package manager in one block selects it in<br>every other package-manager block on the page (persisted in localStorage).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="builtinembedoptions-spotify">
  <td><code>spotify</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;Spotify url=&quot;https://open.spotify.com/track/...&quot;&gt;</code> iframes.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="builtinembedoptions-stackblitz">
  <td><code>stackBlitz</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;StackBlitz url=&quot;https://stackblitz.com/edit/...&quot;&gt;</code> iframes.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="builtinembedoptions-twitter">
  <td><code>twitter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | TwitterEmbedOptions</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;Tweet&gt;</code> / <code>&lt;XPost&gt;</code> as static privacy-conscious cards.<br>Pass <code>{ fetch: true }</code> to fetch the post body, author, and self-hosted<br>media at build time. Fetch failures fall back to the link-only card.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="builtinembedoptions-webcontainer">
  <td><code>webContainer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;WebContainer&gt;</code> lazy placeholders with isolation metadata.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="builtinpmoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BuiltinPmOptions</code><span class="ox-api-entry__description">Options for the package-manager install-tab transform.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for the package-manager install-tab transform.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BuiltinPmOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1384-L1390" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="builtinpmoptions-sync">
  <td><code>sync</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable opt-in synced package-manager tab groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="cardoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CardOptions</code><span class="ox-api-entry__description">Options for opt-in ::: card / ::: link-card / ::: card-grid blocks.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>::: card</code> / <code>::: link-card</code> / <code>::: card-grid</code> blocks.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CardOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1644-L1651" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="cardoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the card transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="cascadeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CascadeOptions</code><span class="ox-api-entry__description">Opt-in _index directory frontmatter cascade. false or omitted stays off. true o…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in <code>_index</code> directory frontmatter cascade.</p>
<p><code>false</code> or omitted stays off. <code>true</code> or <code>{}</code> enables defaults. Set <code>enabled: false</code> on the object to turn the feature back off.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CascadeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L668-L674" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="cascadeoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable directory-level frontmatter inheritance.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="codeannotationkind" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeAnnotationKind = &quot;highlight&quot; | &quot;warning&quot; | &quot;error&quot;</code><span class="ox-api-entry__description">Supported line annotation kinds for code blocks.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Supported line annotation kinds for code blocks.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type CodeAnnotationKind = &quot;highlight&quot; | &quot;warning&quot; | &quot;error&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1957" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="codeannotationsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeAnnotationsOptions</code><span class="ox-api-entry__description">Opt-in code annotation configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in code annotation configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CodeAnnotationsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1967-L1997" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="codeannotationsoptions-defaultlinenumbers">
  <td><code>defaultLineNumbers</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable line numbers for all code blocks by default.<br><br>In <code>vitepress</code> or <code>both</code> mode, fenced code blocks can override this with<br><code>:line-numbers</code>, <code>:line-numbers=&lt;start&gt;</code>, or <code>:no-line-numbers</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="codeannotationsoptions-metakey">
  <td><code>metaKey</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Attribute name read from the code fence meta string.<br><br>Example: <code>annotate=&quot;highlight:1,3-4;warning:6&quot;</code></div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;annotate&quot;</code></div></td>
</tr>
<tr id="codeannotationsoptions-notation">
  <td><code>notation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#codeannotationsyntax">CodeAnnotationSyntax</a></code></td>
  <td><div class="ox-api-entry__member-description">Annotation syntax to enable.<br><br>- <code>attribute</code>: custom attribute syntax like <code>annotate=&quot;highlight:1,3-4&quot;</code><br>- <code>vitepress</code>: VitePress-compatible syntax like <code>{1,3-4}</code> and <code>[!code warning]</code><br>- <code>both</code>: enables both syntaxes</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;attribute&quot;</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="codeannotationsyntax" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeAnnotationSyntax = &quot;attribute&quot; | &quot;vitepress&quot; | &quot;both&quot;</code><span class="ox-api-entry__description">Supported code annotation syntaxes.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Supported code annotation syntaxes.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type CodeAnnotationSyntax = &quot;attribute&quot; | &quot;vitepress&quot; | &quot;both&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1962" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="codeblocklintoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeBlockLintOptions</code><span class="ox-api-entry__description">Options for linting fenced code blocks during Markdown transforms. These checks…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for linting fenced code blocks during Markdown transforms.</p>
<p>These checks are intentionally local to each fence. They do not execute code or parse a project graph, so they are safe to run during normal Markdown transformation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CodeBlockLintOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1811-L1848" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="codeblocklintoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Languages to lint. Omit to lint every fenced block language.<br><br>Language names are compared case-insensitively.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="codeblocklintoptions-mode">
  <td><code>mode</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;warn&quot; | &quot;error&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Diagnostic severity for lint failures.<br><br>Use <code>&#39;error&#39;</code> when code-block lint failures should fail the build.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;warn&#39;</code></div></td>
</tr>
<tr id="codeblocklintoptions-requirelanguage">
  <td><code>requireLanguage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Require every fenced code block to declare a language.<br><br>This is helpful for documentation sites where every example should be<br>highlighted and searchable by language.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="codeblocklintoptions-trailingspaces">
  <td><code>trailingSpaces</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report trailing whitespace inside fenced code blocks.<br><br>The check reports the exact line and column range inside the fence content.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="codeblocktypecheckoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeBlockTypecheckOptions</code><span class="ox-api-entry__description">Options for type-checking TypeScript and TSX fenced code blocks. Type-checking…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for type-checking TypeScript and TSX fenced code blocks.</p>
<p>Type-checking writes matching snippets to a temporary directory and invokes <code>tsgo</code>. It is best suited for concise examples that should stay synchronized with the public TypeScript API.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CodeBlockTypecheckOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1868-L1905" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="codeblocktypecheckoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Fence languages to type-check.<br><br>Language names are compared case-insensitively.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;ts&#39;, &#39;tsx&#39;]</code></div></td>
</tr>
<tr id="codeblocktypecheckoptions-mode">
  <td><code>mode</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;warn&quot; | &quot;error&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Diagnostic severity for type-check failures.<br><br>Use <code>&#39;error&#39;</code> to fail the Markdown transform on broken snippets.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;warn&#39;</code></div></td>
</tr>
<tr id="codeblocktypecheckoptions-requiremeta">
  <td><code>requireMeta</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Require an opt-in fence meta marker before type-checking.<br><br>When enabled, only fences with metadata such as <code>typecheck</code> or <code>twoslash</code><br>are checked.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="codeblocktypecheckoptions-tsgocommand">
  <td><code>tsgoCommand</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Command used to run the TypeScript checker.<br><br>Override this for package-manager scripts or workspace-local binaries.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;tsgo&#39;</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="codeimportoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CodeImportOptions</code><span class="ox-api-entry__description">Options for importing source snippets into code fences. The transform resolves…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for importing source snippets into code fences.</p>
<p>The transform resolves <code>&lt;&lt;&lt;</code> imports before code highlighting and other code-block features run. Imported snippets therefore behave like ordinary fenced code in later stages.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CodeImportOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1592-L1606" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="codeimportoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Directory used to resolve <code>&lt;&lt;&lt;</code> imports.<br><br>When omitted, imports resolve from the Vite project root and configured aliases.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="collectionentry" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CollectionEntry</code><span class="ox-api-entry__description">Queryable Markdown collection entry.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">13 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Queryable Markdown collection entry.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CollectionEntry</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2820-L2834" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group ox-api-entry__member-group--indexable">
<h5>Indexable</h5>
<div class="ox-api-entry__member-details">
<section class="ox-api-entry__member-detail ox-api-entry__member-detail--indexable">
<pre><code class="language-ts">[key: string]: unknown</code></pre></section>
</div>
</div>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="collectionentry-body">
  <td><code>body</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-collection">
  <td><code>collection</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-extension">
  <td><code>extension</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="collectionentry-html">
  <td><code>html</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-id">
  <td><code>id</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-path">
  <td><code>path</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-source">
  <td><code>source</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-stem">
  <td><code>stem</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="collectionentry-toc">
  <td><code>toc</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#tocentry">TocEntry</a>[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="collectionincludefield" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CollectionIncludeField = &quot;body&quot; | &quot;html&quot; | &quot;toc&quot;</code><span class="ox-api-entry__description">Extra payload fields embedded into collection entries. Keep this list small for…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extra payload fields embedded into collection entries.</p>
<p>Keep this list small for large sites. By default collection entries contain only route metadata and frontmatter. <code>body</code>, <code>html</code>, and <code>toc</code> increase the virtual module size, and <code>html</code>/<code>toc</code> require a native Markdown transform.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type CollectionIncludeField = &quot;body&quot; | &quot;html&quot; | &quot;toc&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2767" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="collectionmanifest" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CollectionManifest</code><span class="ox-api-entry__description">Generated collection manifest.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generated collection manifest.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CollectionManifest</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2839-L2841" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="collectionmanifest-collections">
  <td><code>collections</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, <a href="#collectionentry">CollectionEntry</a>[]&gt;</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="collectionoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CollectionOptions</code><span class="ox-api-entry__description">Collection source configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Collection source configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface CollectionOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2772-L2793" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="collectionoptions-include">
  <td><code>include</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">readonly <a href="#collectionincludefield">CollectionIncludeField</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Optional fields to include in each entry.<br><br>The default is metadata-only for performance. Use <code>body</code> for stripped raw<br>Markdown, <code>html</code> for native rendered HTML, and <code>toc</code> for the parsed table<br>of contents.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="collectionoptions-source">
  <td><code>source</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string | readonly string[]</code></td>
  <td><div class="ox-api-entry__member-description">Glob pattern(s) resolved from <code>srcDir</code>.<br><br>Patterns are filtered by the configured Markdown extensions. Numeric route<br>prefixes such as <code>1.guide/2.install.md</code> are stripped from generated <code>path</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">all Markdown files</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="collectionsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">CollectionsOptions = Record&lt;string, CollectionOptions | string | readonly string[]&gt;</code><span class="ox-api-entry__description">Top-level collection definitions.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Top-level collection definitions.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type CollectionsOptions = Record&lt;string, CollectionOptions | string | readonly string[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2798" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="containeroptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ContainerOptions</code><span class="ox-api-entry__description">Options for opt-in ::: type custom containers.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>::: type</code> custom containers.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ContainerOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1428-L1442" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="containeroptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the container transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="containeroptions-types">
  <td><code>types</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, <a href="#containertypeoptions">ContainerTypeOptions</a>&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Extra or overriding container types.<br><br>Keys must be ASCII identifiers (<code>[A-Za-z0-9_-]+</code>). Unknown hostile names<br>are ignored.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="containertypeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ContainerTypeOptions</code><span class="ox-api-entry__description">Per-type container presentation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Per-type container presentation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ContainerTypeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1447-L1452" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="containertypeoptions-tag">
  <td><code>tag</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;div&quot; | &quot;details&quot;</code></td>
  <td><div class="ox-api-entry__member-description"><code>&quot;details&quot;</code> renders <code>&lt;details&gt;</code>/<code>&lt;summary&gt;</code>; anything else is a <code>&lt;div&gt;</code>.</div></td>
</tr>
<tr id="containertypeoptions-title">
  <td><code>title</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Title used when the opener does not set one.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="docentry" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocEntry</code><span class="ox-api-entry__description">A single documentation entry extracted from source. Entries represent top-level…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">14 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A single documentation entry extracted from source.</p>
<p>Entries represent top-level declarations such as functions, classes, interfaces, type aliases, enums, variables, and modules. Members of compound declarations are stored in <code>members</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface DocEntry</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2531-L2573" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="docentry-description">
  <td><code>description</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Main prose extracted from the leading JSDoc/TSDoc block.</div></td>
</tr>
<tr id="docentry-endline">
  <td><code>endLine</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-based end line of the declaration in the source file.</div></td>
</tr>
<tr id="docentry-examples">
  <td><code>examples</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Code examples collected from <code>@example</code> tags.</div></td>
</tr>
<tr id="docentry-file">
  <td><code>file</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source file path relative to the extraction root when available.</div></td>
</tr>
<tr id="docentry-kind">
  <td><code>kind</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;function&quot; | &quot;class&quot; | &quot;interface&quot; | &quot;type&quot; | &quot;enum&quot; | &quot;variable&quot; | &quot;module&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Normalized declaration kind used for grouping and rendering.</div></td>
</tr>
<tr id="docentry-line">
  <td><code>line</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-based start line of the declaration in the source file.</div></td>
</tr>
<tr id="docentry-members">
  <td><code>members</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#docmember">DocMember</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Members belonging to classes, interfaces, object types, and enums.</div></td>
</tr>
<tr id="docentry-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Exported or declared symbol name.</div></td>
</tr>
<tr id="docentry-params">
  <td><code>params</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#paramdoc">ParamDoc</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Function, method, or constructor parameter documentation.</div></td>
</tr>
<tr id="docentry-private">
  <td><code>private</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the entry is marked private or matched by private filtering.</div></td>
</tr>
<tr id="docentry-returns">
  <td><code>returns</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#returndoc">ReturnDoc</a></code></td>
  <td><div class="ox-api-entry__member-description">Return value documentation for callable declarations.</div></td>
</tr>
<tr id="docentry-signature">
  <td><code>signature</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Full declaration signature, when the renderer can extract one.</div></td>
</tr>
<tr id="docentry-tags">
  <td><code>tags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Additional tags preserved by tag name after known tags are normalized.</div></td>
</tr>
<tr id="docentry-throws">
  <td><code>throws</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#throwsdoc">ThrowsDoc</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Exceptions/errors documented with <code>@throws</code> / <code>@exception</code>.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="docmember" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocMember</code><span class="ox-api-entry__description">A member belonging to a class, interface, type alias, or enum entry.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">16 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A member belonging to a class, interface, type alias, or enum entry.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface DocMember</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2578-L2626" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="docmember-default">
  <td><code>default</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Default value extracted from syntax or <code>@default</code> tags.</div></td>
</tr>
<tr id="docmember-description">
  <td><code>description</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Main prose extracted from the member&#39;s documentation comment.</div></td>
</tr>
<tr id="docmember-endline">
  <td><code>endLine</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-based end line of the member declaration.</div></td>
</tr>
<tr id="docmember-kind">
  <td><code>kind</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;property&quot; | &quot;method&quot; | &quot;constructor&quot; | &quot;getter&quot; | &quot;setter&quot; | &quot;enumMember&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Normalized member kind used for rendering and sorting.</div></td>
</tr>
<tr id="docmember-line">
  <td><code>line</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-based start line of the member declaration.</div></td>
</tr>
<tr id="docmember-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Member name as it appears in the containing declaration.</div></td>
</tr>
<tr id="docmember-optional">
  <td><code>optional</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the member is optional in the source declaration.</div></td>
</tr>
<tr id="docmember-params">
  <td><code>params</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#paramdoc">ParamDoc</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Parameter documentation for methods and constructors.</div></td>
</tr>
<tr id="docmember-private">
  <td><code>private</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the member is marked private or matched by private filtering.</div></td>
</tr>
<tr id="docmember-readonly">
  <td><code>readonly</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the member is declared readonly.</div></td>
</tr>
<tr id="docmember-returns">
  <td><code>returns</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#returndoc">ReturnDoc</a></code></td>
  <td><div class="ox-api-entry__member-description">Return value documentation for methods and accessors.</div></td>
</tr>
<tr id="docmember-signature">
  <td><code>signature</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Full member signature, when available.</div></td>
</tr>
<tr id="docmember-static">
  <td><code>static</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the member is static.</div></td>
</tr>
<tr id="docmember-tags">
  <td><code>tags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Additional tags preserved by tag name after known tags are normalized.</div></td>
</tr>
<tr id="docmember-throws">
  <td><code>throws</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#throwsdoc">ThrowsDoc</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Exceptions/errors documented with <code>@throws</code> / <code>@exception</code>.</div></td>
</tr>
<tr id="docmember-type">
  <td><code>type</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered TypeScript type text for properties and enum members.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="docsentrypoint" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocsEntryPoint = string | { path: string; name?: string }</code><span class="ox-api-entry__description">Public API entry point for grouped documentation.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Public API entry point for grouped documentation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type DocsEntryPoint = string | { path: string; name?: string }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2177-L2182" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="docsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocsOptions</code><span class="ox-api-entry__description">Options for source documentation generation. The generator extracts JSDoc/TSDoc…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">33 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for source documentation generation.</p>
<p>The generator extracts JSDoc/TSDoc comments from JavaScript and TypeScript source files, normalizes the declarations, and writes Markdown plus optional navigation metadata. The defaults are optimized for documenting a package&#39;s public <code>src</code> tree without exposing private implementation details.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface DocsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2217-L2483" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="docsoptions-basepath">
  <td><code>basePath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Route prefix used by generated documentation links and nav metadata.<br><br>Nav metadata falls back to <code>/api</code> when this is not set.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-classpropertiesformat">
  <td><code>classPropertiesFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for class property groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable source documentation generation.<br><br>The top-level <code>docs</code> option is opt-out: omitting it enables docs generation<br>with defaults, while <code>docs: false</code> disables the docs plugin entirely.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="docsoptions-entrypoints">
  <td><code>entryPoints</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#docsentrypoint">DocsEntryPoint</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Public API entry points used to group re-exported docs.<br><br>When omitted, docs are generated from the discovered source files without<br>entry-point grouping.<br><br>Use entry points when a package exposes a smaller public surface than its<br>source tree. Re-exported declarations are grouped under the entry point that<br>exposes them.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-enummembersformat">
  <td><code>enumMembersFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for enum member groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-exclude">
  <td><code>exclude</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Glob patterns for files to exclude.<br><br>Excludes run after <code>include</code> matching and should cover tests, generated<br>files, and implementation-only entry points.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;**\/*.test.*&#39;, &#39;**\/*.spec.*&#39;, &#39;node_modules&#39;]</code></div></td>
</tr>
<tr id="docsoptions-format">
  <td><code>format</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;markdown&quot; | &quot;json&quot; | &quot;html&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Output format.<br><br><code>markdown</code> is the primary supported format. <code>json</code> and <code>html</code> are reserved<br>for consumers that want to post-process extracted documentation data.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;markdown&#39;</code></div></td>
</tr>
<tr id="docsoptions-generatenav">
  <td><code>generateNav</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Generate navigation metadata file.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="docsoptions-githuburl">
  <td><code>githubUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">GitHub repository URL for source code links.<br><br>When provided, generated documentation includes links back to the source<br>declaration lines.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-groupby">
  <td><code>groupBy</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;file&quot; | &quot;category&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Group documentation by file or category.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;file&#39;</code></div></td>
</tr>
<tr id="docsoptions-grouporder">
  <td><code>groupOrder</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">TypeDoc-style group order for module index sections and nav groups.<br>Use <code>*</code> as the insertion point for unlisted groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-include">
  <td><code>include</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Glob patterns for files to include.<br><br>Patterns are evaluated inside each <code>src</code> directory.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;**\/*.ts&#39;, &#39;**\/*.tsx&#39;, &#39;**\/*.js&#39;, &#39;**\/*.jsx&#39;, &#39;**\/*.mts&#39;, &#39;**\/*.mjs&#39;, &#39;**\/*.cts&#39;, &#39;**\/*.cjs&#39;]</code></div></td>
</tr>
<tr id="docsoptions-indexformat">
  <td><code>indexFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for index items.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-interfacepropertiesformat">
  <td><code>interfacePropertiesFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for interface property groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-internal">
  <td><code>internal</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Include internal members in documentation.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="docsoptions-kindsortorder">
  <td><code>kindSortOrder</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">TypeDoc-style declaration kind ranking for module sections and nav groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-linkstyle">
  <td><code>linkStyle</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;markdown&quot; | &quot;clean&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Internal documentation link style.<br><br>Use <code>markdown</code> for generated <code>.md</code> targets and <code>clean</code> for route-style links<br>consumed by static-site frameworks.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;markdown&#39;</code></div></td>
</tr>
<tr id="docsoptions-out">
  <td><code>out</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Output directory for generated documentation.<br><br>The path is resolved from the Vite project root. Markdown pages, <code>docs.json</code>,<br>and generated navigation metadata are written under this directory.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;docs/api&#39;</code></div></td>
</tr>
<tr id="docsoptions-parametersformat">
  <td><code>parametersFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for value and type parameters.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-pathstrategy">
  <td><code>pathStrategy</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;flat&quot; | &quot;typedoc&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Generated Markdown output path strategy.<br><br><code>flat</code> emits one page per source module or category. <code>typedoc</code> emits<br>TypeDoc-like module, kind, and symbol pages for larger API references.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;flat&#39;</code></div></td>
</tr>
<tr id="docsoptions-private">
  <td><code>private</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Include private members in documentation.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="docsoptions-propertymembersformat">
  <td><code>propertyMembersFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for property-owned object literal members.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-rendergeneratedby">
  <td><code>renderGeneratedBy</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Emit the generated-by attribution on generated root index pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="docsoptions-renderstats">
  <td><code>renderStats</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Emit the stats summary line on generated index pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="docsoptions-renderstyle">
  <td><code>renderStyle</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;html&quot; | &quot;markdown&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Rendering style for generated API Markdown.<br><br>- <code>&#39;html&#39;</code> (default): HTML-laced Markdown with collapsible entries, stat<br>  blocks and member tables (ox-content theme).<br>- <code>&#39;markdown&#39;</code>: pure Markdown (headings, tables, fenced code) with no raw<br>  HTML scaffolding, suitable for plain Markdown hosts such as VitePress.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;html&#39;</code></div></td>
</tr>
<tr id="docsoptions-singleentryroot">
  <td><code>singleEntryRoot</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;preserve&quot; | &quot;flatten&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Single-entry root handling for TypeDoc-style generated docs.<br><br>When set to <code>&#39;flatten&#39;</code>, a single TypeDoc entry point uses the root<br><code>index.md</code> as its landing page and omits the extra module level from<br>generated nav metadata. Symbol page paths stay under the entry point.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;preserve&#39;</code></div></td>
</tr>
<tr id="docsoptions-sort">
  <td><code>sort</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">DocsSortStrategy[]</code></td>
  <td><div class="ox-api-entry__member-description">TypeDoc-style sort strategies applied to entries and members.<br>Strategies run in order; later strategies break ties from earlier ones.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="docsoptions-sortentrypoints">
  <td><code>sortEntryPoints</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Preserve caller-provided entry point order when false.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="docsoptions-src">
  <td><code>src</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Source directories to scan for documentation.<br><br>Paths are resolved from the Vite project root before applying <code>include</code> and<br><code>exclude</code> patterns.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;./src&#39;]</code></div></td>
</tr>
<tr id="docsoptions-toc">
  <td><code>toc</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Generate table of contents for each file.<br>Reserved for future use; current generated API pages do not emit this TOC.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="docsoptions-typealiaspropertiesformat">
  <td><code>typeAliasPropertiesFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for type alias property groups.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-typedeclarationformat">
  <td><code>typeDeclarationFormat</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td><div class="ox-api-entry__member-description">Display format for return type declaration members.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;none&#39;</code></div></td>
</tr>
<tr id="docsoptions-typeparameters">
  <td><code>typeParameters</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Opt in to TSDoc-style type-parameter documentation.<br><br>When enabled, declaration type parameters (<code>&lt;T extends C = D&gt;</code>) are<br>extracted into a structured &quot;Type Parameters&quot; section and <code>@typeParam</code> /<br><code>@template</code> tags are merged in (and removed from the generic tag list).<br><code>@typeParam</code> is a TSDoc feature, so this is off by default (JSDoc semantics).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="docssummary" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocsSummary</code><span class="ox-api-entry__description">Summary counts emitted with generated documentation data.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Summary counts emitted with generated documentation data.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface DocsSummary</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2696-L2717" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="docssummary-bykind">
  <td><code>byKind</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, number&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Entry counts grouped by normalized declaration kind.</div></td>
</tr>
<tr id="docssummary-deprecated">
  <td><code>deprecated</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of entries or members marked with <code>@deprecated</code>.</div></td>
</tr>
<tr id="docssummary-entries">
  <td><code>entries</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of top-level entries across all modules.</div></td>
</tr>
<tr id="docssummary-examples">
  <td><code>examples</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of collected examples.</div></td>
</tr>
<tr id="docssummary-modules">
  <td><code>modules</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of modules included in the generated payload.</div></td>
</tr>
<tr id="docssummary-params">
  <td><code>params</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of documented parameters.</div></td>
</tr>
<tr id="docssummary-returns">
  <td><code>returns</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of documented return values.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="docstestoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DocsTestOptions</code><span class="ox-api-entry__description">Options for extracting fenced examples into docs-as-tests fixtures. The extract…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for extracting fenced examples into docs-as-tests fixtures.</p>
<p>The extractor collects code fences that can be written into test files and executed by the exported docs test harness helpers.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface DocsTestOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1924-L1943" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="docstestoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Fence languages to collect as runnable examples.<br><br>Language names are compared case-insensitively.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;js&#39;, &#39;jsx&#39;, &#39;ts&#39;, &#39;tsx&#39;]</code></div></td>
</tr>
<tr id="docstestoptions-requiremeta">
  <td><code>requireMeta</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Require an opt-in fence meta marker before collecting an example.<br><br>When enabled, only fences marked with metadata such as <code>test</code>, <code>runnable</code>,<br><code>vitest</code>, or <code>docs-test</code> are collected.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="editthispageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">EditThisPageOptions</code><span class="ox-api-entry__description">Options for appending an &quot;edit this page&quot; link. The generated link points at th…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for appending an &quot;edit this page&quot; link.</p>
<p>The generated link points at the source Markdown file rather than the emitted HTML route. Configure <code>branch</code> and <code>rootDir</code> to match the repository layout users should edit.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface EditThisPageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1751-L1791" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="editthispageoptions-branch">
  <td><code>branch</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Branch used in generated edit links.<br><br>Use the branch that accepts documentation changes, not necessarily the<br>branch that produced the deployed site.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;main&#39;</code></div></td>
</tr>
<tr id="editthispageoptions-label">
  <td><code>label</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Link text rendered in the page footer.<br><br>Keep this short; the default theme renders it as a compact footer action.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;Edit this page&#39;</code></div></td>
</tr>
<tr id="editthispageoptions-repourl">
  <td><code>repoUrl</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Repository URL used to build edit links.<br><br>The transform is enabled only when this value is provided.</div></td>
</tr>
<tr id="editthispageoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source root inside the repository, used before the page path.<br><br>Set this when <code>srcDir</code> is nested in a package or docs workspace.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="emojishortcodeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">EmojiShortcodeOptions</code><span class="ox-api-entry__description">Options for expanding :shortcode: emoji aliases. The transform replaces recogni…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for expanding <code>:shortcode:</code> emoji aliases.</p>
<p>The transform replaces recognized shortcode tokens with their Unicode emoji equivalents during Markdown transformation. Unknown shortcodes are left untouched so colon-delimited text can still be used by other tools.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface EmojiShortcodeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1516-L1530" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="emojishortcodeoptions-custom">
  <td><code>custom</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Custom shortcode map merged with the built-in emoji aliases.<br><br>Keys should omit the surrounding colons.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{}</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="entrypageconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">EntryPageConfig</code><span class="ox-api-entry__description">Entry page frontmatter configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Entry page frontmatter configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface EntryPageConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L107-L116" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="entrypageconfig-features">
  <td><code>features</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#featureconfig">FeatureConfig</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Feature cards</div></td>
</tr>
<tr id="entrypageconfig-hero">
  <td><code>hero</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#heroconfig">HeroConfig</a></code></td>
  <td><div class="ox-api-entry__member-description">Hero section</div></td>
</tr>
<tr id="entrypageconfig-layout">
  <td><code>layout</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;entry&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Layout type - set to &#39;entry&#39; for entry page</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="extracteddocs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ExtractedDocs</code><span class="ox-api-entry__description">Extracted documentation for a single file.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extracted documentation for a single file.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ExtractedDocs</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2673-L2691" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="extracteddocs-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Optional module-level description extracted from a file header comment.</div></td>
</tr>
<tr id="extracteddocs-entries">
  <td><code>entries</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#docentry">DocEntry</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Top-level documented declarations found in this module.</div></td>
</tr>
<tr id="extracteddocs-examples">
  <td><code>examples</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Module-level examples collected from a file header comment.</div></td>
</tr>
<tr id="extracteddocs-file">
  <td><code>file</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source module or file identifier used by generated output.</div></td>
</tr>
<tr id="extracteddocs-sourcepath">
  <td><code>sourcePath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Absolute source path, when available for source links and diagnostics.</div></td>
</tr>
<tr id="extracteddocs-tags">
  <td><code>tags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Module-level tags preserved by tag name.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="featureconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">FeatureConfig</code><span class="ox-api-entry__description">Feature card for entry page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Feature card for entry page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface FeatureConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L87-L102" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="featureconfig-details">
  <td><code>details</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Feature description</div></td>
</tr>
<tr id="featureconfig-icon">
  <td><code>icon</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Icon - supports: &quot;mdi:icon-name&quot; (Iconify), image URL, or emoji</div></td>
</tr>
<tr id="featureconfig-link">
  <td><code>link</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Optional link</div></td>
</tr>
<tr id="featureconfig-linktext">
  <td><code>linkText</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Link text</div></td>
</tr>
<tr id="featureconfig-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Feature title</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="feedformat" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">FeedFormat = &quot;rss&quot; | &quot;atom&quot; | &quot;json&quot;</code><span class="ox-api-entry__description">Feed file formats written during SSG.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Feed file formats written during SSG.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type FeedFormat = &quot;rss&quot; | &quot;atom&quot; | &quot;json&quot;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L738" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="feedsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">FeedsOptions</code><span class="ox-api-entry__description">Opt-in RSS / Atom / JSON Feed files written during SSG.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in RSS / Atom / JSON Feed files written during SSG.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface FeedsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L743-L767" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="feedsoptions-collection">
  <td><code>collection</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Named collection to publish. Defaults to <code>content</code>, or the first<br>configured collection when <code>content</code> is absent.</div></td>
</tr>
<tr id="feedsoptions-formats">
  <td><code>formats</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#feedformat">FeedFormat</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Feed formats to write.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&quot;rss&quot;, &quot;atom&quot;, &quot;json&quot;]</code></div></td>
</tr>
<tr id="feedsoptions-limit">
  <td><code>limit</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Maximum number of published items, newest first.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">20</code></div></td>
</tr>
<tr id="feedsoptions-path">
  <td><code>path</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site-relative directory for the generated files.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;/&quot;</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="filetreeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">FileTreeOptions</code><span class="ox-api-entry__description">Options for opt-in file-tree fences.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>file-tree</code> fences.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface FileTreeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1682-L1689" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="filetreeoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the file-tree transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="generateddocsdata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">GeneratedDocsData</code><span class="ox-api-entry__description">Machine-readable payload emitted alongside generated docs.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Machine-readable payload emitted alongside generated docs.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface GeneratedDocsData</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2722-L2734" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="generateddocsdata-generatedat">
  <td><code>generatedAt</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">ISO timestamp for the generation run.</div></td>
</tr>
<tr id="generateddocsdata-modules">
  <td><code>modules</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#extracteddocs">ExtractedDocs</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Extracted documentation modules in render order.</div></td>
</tr>
<tr id="generateddocsdata-summary">
  <td><code>summary</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#docssummary">DocsSummary</a></code></td>
  <td><div class="ox-api-entry__member-description">Aggregate counts useful for dashboards and generated index pages.</div></td>
</tr>
<tr id="generateddocsdata-version">
  <td><code>version</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">1</code></td>
  <td><div class="ox-api-entry__member-description">Payload schema version. Increment when the JSON shape changes incompatibly.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="heroaction" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">HeroAction</code><span class="ox-api-entry__description">Hero section action button.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Hero section action button.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface HeroAction</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L16-L25" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="heroaction-link">
  <td><code>link</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Link URL</div></td>
</tr>
<tr id="heroaction-text">
  <td><code>text</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Button text</div></td>
</tr>
<tr id="heroaction-theme">
  <td><code>theme</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;brand&quot; | &quot;alt&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Button theme: &#39;brand&#39; (primary) or &#39;alt&#39; (secondary)</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="heroconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">HeroConfig</code><span class="ox-api-entry__description">Hero section configuration for entry page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Hero section configuration for entry page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface HeroConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L64-L82" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="heroconfig-actions">
  <td><code>actions</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#heroaction">HeroAction</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Action buttons</div></td>
</tr>
<tr id="heroconfig-image">
  <td><code>image</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#heroimage">HeroImage</a></code></td>
  <td><div class="ox-api-entry__member-description">Hero image</div></td>
</tr>
<tr id="heroconfig-name">
  <td><code>name</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Main title (large, gradient text)</div></td>
</tr>
<tr id="heroconfig-notice">
  <td><code>notice</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#heronotice">HeroNotice</a></code></td>
  <td><div class="ox-api-entry__member-description">Notice shown near the top of the hero</div></td>
</tr>
<tr id="heroconfig-tagline">
  <td><code>tagline</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Tagline (smaller, muted)</div></td>
</tr>
<tr id="heroconfig-text">
  <td><code>text</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Secondary text (medium size)</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="heroimage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">HeroImage</code><span class="ox-api-entry__description">Hero section image configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Hero section image configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface HeroImage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L30-L48" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="heroimage-alt">
  <td><code>alt</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Alt text</div></td>
</tr>
<tr id="heroimage-darksrc">
  <td><code>darkSrc</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Dark mode image source URL</div></td>
</tr>
<tr id="heroimage-height">
  <td><code>height</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image height</div></td>
</tr>
<tr id="heroimage-lightsrc">
  <td><code>lightSrc</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Light mode image source URL</div></td>
</tr>
<tr id="heroimage-src">
  <td><code>src</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Image source URL</div></td>
</tr>
<tr id="heroimage-width">
  <td><code>width</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image width</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="heronotice" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">HeroNotice</code><span class="ox-api-entry__description">Hero notice configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Hero notice configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface HeroNotice</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L53-L59" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="heronotice-body">
  <td><code>body</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Notice paragraphs</div></td>
</tr>
<tr id="heronotice-title">
  <td><code>title</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Notice title</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="i18noptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">I18nOptions</code><span class="ox-api-entry__description">i18n (internationalization) options. i18n is opt-in because it changes routing…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>i18n (internationalization) options.</p>
<p>i18n is opt-in because it changes routing and build-time validation. Set <code>enabled: true</code> and configure at least <code>defaultLocale</code> / <code>locales</code> when the same content tree should serve multiple languages.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface I18nOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L3040-L3108" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="i18noptions-check">
  <td><code>check</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Run i18n checks during build.<br><br>Checks validate dictionary coverage and translation function usage when the<br>native i18n checker is available.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="i18noptions-defaultlocale">
  <td><code>defaultLocale</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Default locale tag.<br><br>The default locale is added to <code>locales</code> automatically when omitted from the<br>list.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;en&#39;</code></div></td>
</tr>
<tr id="i18noptions-dir">
  <td><code>dir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Path to i18n dictionary directory (relative to project root).<br><br>Dictionary files are watched in development and checked during builds when<br><code>check</code> is enabled.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;content/i18n&#39;</code></div></td>
</tr>
<tr id="i18noptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable i18n.<br><br>The resolver returns <code>false</code> unless this is explicitly set to <code>true</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="i18noptions-functionnames">
  <td><code>functionNames</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Translation function names to detect in source code.<br><br>Add framework-specific wrappers here so build-time checks can find all<br>translation keys.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;t&#39;, &#39;$t&#39;]</code></div></td>
</tr>
<tr id="i18noptions-hidedefaultlocale">
  <td><code>hideDefaultLocale</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Hide default locale prefix in URLs.<br><br>When true, <code>/page</code> serves the default locale and <code>/ja/page</code> serves Japanese.<br>When false, all locales get prefixed: <code>/en/page</code>, <code>/ja/page</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="i18noptions-locales">
  <td><code>locales</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#localeconfig">LocaleConfig</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Available locales.<br><br>When omitted, ox-content creates a single locale from <code>defaultLocale</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[{ code: defaultLocale, name: defaultLocale }]</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="imageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ImageOptions</code><span class="ox-api-entry__description">Options for opt-in figures, captions, and lazy images.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in figures, captions, and lazy images.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1465-L1472" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="imageoptions-lazy">
  <td><code>lazy</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Add <code>loading=&quot;lazy&quot;</code> to transformed images.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="includeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">IncludeOptions</code><span class="ox-api-entry__description">Options for inlining Markdown files with &lt;!-- @include: PATH --&gt;. Relative path…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for inlining Markdown files with <code>&lt;!-- @include: PATH --&gt;</code>.</p>
<p>Relative paths resolve from the current file. <code>@/</code> and leading <code>/</code> resolve from <code>rootDir</code>. After canonicalize, paths outside <code>rootDir</code> are rejected.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface IncludeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1622-L1631" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="includeoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Directory used to resolve <code>@/</code> and absolute include paths.<br><br>When omitted, includes resolve from the Vite project root.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="localeconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">LocaleConfig</code><span class="ox-api-entry__description">Locale configuration. Locales define the routing and display metadata used by t…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Locale configuration.</p>
<p>Locales define the routing and display metadata used by the i18n plugin.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface LocaleConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L3018-L3031" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="localeconfig-code">
  <td><code>code</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">BCP 47 locale tag (e.g., &#39;en&#39;, &#39;ja&#39;, &#39;zh-Hans&#39;).</div></td>
</tr>
<tr id="localeconfig-dir">
  <td><code>dir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;ltr&quot; | &quot;rtl&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Text direction for rendered pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;ltr&#39;</code></div></td>
</tr>
<tr id="localeconfig-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Display name for this locale (e.g., &#39;English&#39;, &#39;日本語&#39;).</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownnode" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownNode</code><span class="ox-api-entry__description">Markdown AST node (simplified for TypeScript).</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Markdown AST node (simplified for TypeScript).</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2108-L2113" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group ox-api-entry__member-group--indexable">
<h5>Indexable</h5>
<div class="ox-api-entry__member-details">
<section class="ox-api-entry__member-detail ox-api-entry__member-detail--indexable">
<pre><code class="language-ts">[key: string]: unknown</code></pre></section>
</div>
</div>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownnode-children">
  <td><code>children</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdownnode">MarkdownNode</a>[]</code></td>
  <td></td>
</tr>
<tr id="markdownnode-type">
  <td><code>type</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="markdownnode-value">
  <td><code>value</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdowntransformer" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownTransformer</code><span class="ox-api-entry__description">Custom AST transformer.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Custom AST transformer.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownTransformer</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2073-L2083" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdowntransformer-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Transformer name.</div></td>
</tr>
<tr id="markdowntransformer-transform">
  <td><code>transform</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">(ast: <a href="#markdownnode">MarkdownNode</a>, context: <a href="#transformcontext">TransformContext</a>) =&gt; <a href="#markdownnode">MarkdownNode</a> | Promise&lt;<a href="#markdownnode">MarkdownNode</a>&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Transform function.</div><ul class="ox-api-entry__member-params"><li><code>ast</code></li><li><code>context</code></li></ul></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="mathoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MathOptions</code><span class="ox-api-entry__description">Options for opt-in $…$ / $$…$$ math.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>$…$</code> / <code>$$…$$</code> math.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MathOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1543-L1550" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="mathoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the math transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="navitem" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NavItem</code><span class="ox-api-entry__description">Navigation item for sidebar navigation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Navigation item for sidebar navigation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NavItem</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2739-L2754" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="navitem-children">
  <td><code>children</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#navitem">NavItem</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Child navigation items (optional).</div></td>
</tr>
<tr id="navitem-path">
  <td><code>path</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Path to the documentation page.</div></td>
</tr>
<tr id="navitem-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Display title for the navigation item.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="notfoundoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NotFoundOptions</code><span class="ox-api-entry__description">Opt-in custom 404 page written during SSG.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in custom 404 page written during SSG.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NotFoundOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L517-L529" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="notfoundoptions-output">
  <td><code>output</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Output file relative to <code>outDir</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;404.html&quot;</code></div></td>
</tr>
<tr id="notfoundoptions-source">
  <td><code>source</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Markdown source relative to <code>srcDir</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;404.md&quot;</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ogimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageOptions</code><span class="ox-api-entry__description">OG image generation options. Uses Chromium-based rendering with customizable te…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image generation options. Uses Chromium-based rendering with customizable templates.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2013-L2056" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimageoptions-cache">
  <td><code>cache</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable content-hash based caching.<br>Skips rendering when content hasn&#39;t changed.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="ogimageoptions-concurrency">
  <td><code>concurrency</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of concurrent page instances for parallel rendering.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">1</code></div></td>
</tr>
<tr id="ogimageoptions-height">
  <td><code>height</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image height in pixels.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">630</code></div></td>
</tr>
<tr id="ogimageoptions-template">
  <td><code>template</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Path to a custom template file (.ts, .vue, .svelte, .tsx/.jsx).<br>- <code>.ts</code>: default-export a function <code>(props) =&gt; string</code><br>- <code>.vue</code>: Vue SFC, rendered via SSR<br>- <code>.svelte</code>: Svelte SFC, rendered via SSR<br>- <code>.tsx</code>/<code>.jsx</code>: React Server Component, rendered via SSR<br>If not specified, the built-in default template is used.</div></td>
</tr>
<tr id="ogimageoptions-vueplugin">
  <td><code>vuePlugin</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;vitejs&quot; | &quot;vizejs&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Vue plugin to use for compiling <code>.vue</code> templates.<br>- <code>&#39;vitejs&#39;</code>: Use <code>@vue/compiler-sfc</code> (official, default)<br>- <code>&#39;vizejs&#39;</code>: Use <code>@vizejs/vite-plugin</code> (Rust-based)</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;vitejs&#39;</code></div></td>
</tr>
<tr id="ogimageoptions-width">
  <td><code>width</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image width in pixels.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">1200</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="oxcontentoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OxContentOptions</code><span class="ox-api-entry__description">Options for the core oxContent() Vite plugin. The top-level options describe wh…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">50 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for the core <code>oxContent()</code> Vite plugin.</p>
<p>The top-level options describe where content lives, which Markdown features are enabled, and which build-time features should run. Feature toggles that accept <code>boolean | Options</code> follow the same convention:</p>
<ul>
<li><code>false</code> disables the feature.</li>
<li><code>true</code> enables the feature with its documented defaults.</li>
<li>an object enables the feature and overrides only the provided fields.</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OxContentOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L791-L1261" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="oxcontentoptions-attrs">
  <td><code>attrs</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#attrsoptions">AttrsOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Enable markdown-it-attrs style <code>{#id .class key=value}</code> attributes.<br><br>Attribute blocks can be attached to headings, paragraphs, links, images, and<br>other supported Markdown nodes depending on parser context.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-autolinks">
  <td><code>autolinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable GFM autolinks and linkify bare URLs.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-badges">
  <td><code>badges</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#badgeoptions">BadgeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in <code>{badge:variant}</code> inline badges.<br><br>Passing <code>true</code> or an options object enables the built-in variants.<br>Badge text is HTML-escaped. Fenced, indented, and inline code are skipped.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-base">
  <td><code>base</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base path prepended to generated internal URLs.<br><br>Use this when the site is deployed below a sub-path, such as GitHub Pages or<br>a documentation route inside a larger application.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;/&#39;</code></div></td>
</tr>
<tr id="oxcontentoptions-cards">
  <td><code>cards</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#cardoptions">CardOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in <code>::: card</code> / <code>::: link-card</code> / <code>::: card-grid</code> blocks.<br><br>Passing <code>true</code> enables the defaults. Pass an object to keep the option<br>shape while overriding <code>enabled</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-cascade">
  <td><code>cascade</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#cascadeoptions">CascadeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Inherit missing frontmatter keys from ancestor <code>_index</code> files.<br><br>Off by default. <code>true</code> or <code>{}</code> fills keys a child does not set.<br><code>permalink</code> and <code>slug</code> are never inherited.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-cjkemphasis">
  <td><code>cjkEmphasis</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Recognize emphasis adjacent to CJK text. The native parser already supports<br>this behavior; the option documents the compatibility contract.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-codeannotations">
  <td><code>codeAnnotations</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#codeannotationsoptions">CodeAnnotationsOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Code block line annotations for fenced code blocks.<br><br>This feature is opt-in because it changes rendered code-block markup. Pass<br><code>true</code> to enable ox-content&#39;s attribute syntax, or pass an options object to<br>change the meta key or enable VitePress-compatible notation.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-codeblocklint">
  <td><code>codeBlockLint</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#codeblocklintoptions">CodeBlockLintOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Lint fenced code blocks during Markdown transforms.<br><br>Use this as a lightweight authoring check for missing languages or trailing<br>whitespace inside fences. For project-wide linting, prefer the exported<br><code>lintCodeBlocks()</code> helper or the Markdown lint APIs.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-codeblocktypecheck">
  <td><code>codeBlockTypecheck</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#codeblocktypecheckoptions">CodeBlockTypecheckOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Type-check TypeScript/TSX code fences via tsgo.<br><br>By default only fences with explicit opt-in metadata are checked. This keeps<br>incidental examples cheap while allowing docs-as-code snippets to fail the<br>build when configured with <code>mode: &#39;error&#39;</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-codeimports">
  <td><code>codeImports</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#codeimportoptions">CodeImportOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Import source snippets into fences with <code>&lt;&lt;&lt; @/path/to/file.ts{region}</code>.<br><br>This is useful for documentation that must stay synchronized with examples<br>in the repository. Use <code>rootDir</code> when snippets should resolve from a<br>directory other than the Vite project root.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-collections">
  <td><code>collections</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#collectionsoptions">CollectionsOptions</a> | boolean</code></td>
  <td><div class="ox-api-entry__member-description">Markdown collection query options.<br><br>Collections are exposed through <code>virtual:ox-content/collections</code>. The<br>default collection is metadata-only and is built by the native Rust<br>manifest builder without rendering every document; add <code>include</code> fields<br>only for routes that need raw or rendered content in the query payload.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">content collection for all Markdown files</code></div></td>
</tr>
<tr id="oxcontentoptions-containers">
  <td><code>containers</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#containeroptions">ContainerOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in <code>::: tip</code> custom containers.<br><br>GitHub-style <code>&gt; [!NOTE]</code> callouts stay available without this option.<br>Passing <code>true</code> enables the built-in types. Pass an object to register extra<br>types or override titles.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-docs">
  <td><code>docs</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#docsoptions">DocsOptions</a> | false</code></td>
  <td><div class="ox-api-entry__member-description">Source documentation generation options.<br>Set to false to disable (opt-out).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{ enabled: true }</code></div></td>
</tr>
<tr id="oxcontentoptions-docstests">
  <td><code>docsTests</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#docstestoptions">DocsTestOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Extract runnable fenced examples for Vitest docs-as-tests harnesses.<br><br>Collected examples can be written by the docs test helpers and executed as<br>part of a normal Vitest suite.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-editthispage">
  <td><code>editThisPage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#editthispageoptions">EditThisPageOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Append an &quot;edit this page&quot; link to rendered Markdown.<br><br>The feature is enabled only when <code>repoUrl</code> is provided in the options object.<br>Passing <code>true</code> keeps the feature disabled because there is not enough<br>repository information to generate valid links.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-embeds">
  <td><code>embeds</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#builtinembedoptions">BuiltinEmbedOptions</a> | false</code></td>
  <td><div class="ox-api-entry__member-description">Built-in static embeds rendered during Markdown transformation.<br>Set to <code>false</code> to disable all built-in embeds.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{ github: true, openGraph: true }</code></div></td>
</tr>
<tr id="oxcontentoptions-emojishortcodes">
  <td><code>emojiShortcodes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#emojishortcodeoptions">EmojiShortcodeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Expand <code>:shortcode:</code> emoji aliases to Unicode.<br><br>Built-in aliases cover common emoji names. Provide <code>custom</code> entries for<br>project-specific aliases or to override a built-in mapping.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-extensions">
  <td><code>extensions</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Markdown-like file extensions to process.<br><br>Extensions are normalized with a leading dot and matched case-insensitively.<br>Add custom extensions when another authoring format is compiled to Markdown<br>before ox-content sees it.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;.md&#39;, &#39;.markdown&#39;, &#39;.mdx&#39;]</code></div></td>
</tr>
<tr id="oxcontentoptions-feeds">
  <td><code>feeds</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#feedsoptions">FeedsOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Write RSS, Atom, and/or JSON Feed files from a named collection.<br><br>Off by default. <code>true</code> writes all three formats from the <code>content</code><br>collection (or the first configured collection) with a 20-item limit.<br>An object enables the feature and overrides only the fields you set.<br>Requires <code>ssg.siteUrl</code>. When that is missing the build continues and a<br>warning is emitted instead of writing files.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-filetree">
  <td><code>fileTree</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#filetreeoptions">FileTreeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in static directory trees from <code>file-tree</code> fences.<br><br>Passing <code>true</code> or <code>{}</code> enables the transform. Names are escaped and never<br>read from the filesystem.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-footnotes">
  <td><code>footnotes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable footnotes.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-frontmatter">
  <td><code>frontmatter</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Parse YAML frontmatter.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-gfm">
  <td><code>gfm</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable GitHub Flavored Markdown extensions.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-highlight">
  <td><code>highlight</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable syntax highlighting for code blocks.<br><br>When true, fenced and language-tagged inline code is highlighted with the<br>native tree-sitter engine. Token colors are <code>--octc-shiki-*</code> custom<br>properties (the <code>shiki</code> prefix is historical) so theme-color packages keep<br>working. Languages with no native grammar stay unhighlighted.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-i18n">
  <td><code>i18n</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#i18noptions">I18nOptions</a> | false</code></td>
  <td><div class="ox-api-entry__member-description">i18n (internationalization) options.<br>Set to false to disable i18n.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-images">
  <td><code>images</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#imageoptions">ImageOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in figures, captions, and lazy-loaded images.<br><br>Title text becomes a <code>&lt;figcaption&gt;</code>. Optional <code>{width=N height=M}</code> on the<br>image is consumed by this feature and does not require <code>attrs</code>. Passing<br><code>true</code> or <code>{}</code> enables defaults (<code>lazy: true</code>).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-includes">
  <td><code>includes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#includeoptions">IncludeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Inline another Markdown file with <code>&lt;!-- @include: ./path.md --&gt;</code>.<br><br>Expansion happens before Markdown is parsed, so included headings and<br>lists become part of the host document. Relative paths resolve from the<br>current file. <code>@/</code> and <code>/</code> resolve from <code>rootDir</code>. Paths outside<br><code>rootDir</code> are rejected and reported as transform errors.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-math">
  <td><code>math</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#mathoptions">MathOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Enable <code>$…$</code> inline and <code>$$…$$</code> block math.<br><br>Currency-like <code>$</code> runs, fenced code, indented code, and inline code stay<br>literal. TeX is HTML-escaped into accessible MathML <code>mtext</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-mermaid">
  <td><code>mermaid</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable mermaid diagram rendering.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-ogimage">
  <td><code>ogImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable OG image generation.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-ogimageoptions">
  <td><code>ogImageOptions</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ogimageoptions">OgImageOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">OG image generation options.<br>Ignored unless <code>ogImage</code> or <code>ssg.generateOgImage</code> is enabled.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{ vuePlugin: &#39;vitejs&#39;, width: 1200, height: 630, cache: true, concurrency: 1 }</code></div></td>
</tr>
<tr id="oxcontentoptions-ogviewer">
  <td><code>ogViewer</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable OG Viewer dev tool.<br>Accessible at /__og-viewer during development.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-outdir">
  <td><code>outDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Directory where generated files are written.<br><br>SSG HTML, search indexes, and generated assets are emitted under this<br>directory during production builds.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;dist&#39;</code></div></td>
</tr>
<tr id="oxcontentoptions-permalinks">
  <td><code>permalinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#permalinksoptions">PermalinksOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Honor frontmatter <code>permalink</code> / <code>slug</code> when resolving page URLs.<br><br>Off by default. <code>true</code> or <code>{}</code> replaces the file-tree URL with<br><code>permalink</code>, or the last path segment with <code>slug</code>. Path escape<br>(<code>../</code>, absolute filesystem paths, <code>javascript:</code>, protocol-relative<br><code>//</code>) is rejected and the file-tree URL is kept. Two pages that<br>resolve to the same URL produce an error; the first page is kept and<br>the later page is skipped.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-publishstate">
  <td><code>publishState</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#publishstateoptions">PublishStateOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Honor frontmatter draft / unlisted / scheduled publish states.<br><br>Off by default. <code>true</code> omits drafts and future-scheduled pages from<br>production HTML, search, and sitemaps. Unlisted pages still build and<br>remain reachable by URL. An object enables the feature and can inject<br><code>now</code> for a deterministic build-time clock.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-redirects">
  <td><code>redirects</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#redirectsoptions">RedirectsOptions</a> | Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Write static HTML redirect pages for frontmatter aliases and a config map.<br><br>Off by default. <code>true</code> or <code>{}</code> enables empty defaults. A path map such as<br><code>{ &quot;/old-guide&quot;: &quot;/guide&quot; }</code> enables the feature with that map. Destinations<br>must be same-origin paths (<code>/</code> but not <code>//</code>) unless <code>allowExternal</code> is set.<br><code>javascript:</code>, <code>data:</code>, and protocol-relative URLs are ignored.<br>Overlapping sources last-win after trailing slashes are folded.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-sanitize">
  <td><code>sanitize</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#sanitizeoptions">SanitizeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Sanitize rendered HTML with safe defaults or explicit allow lists.<br><br>Enable this for untrusted Markdown. The default allow lists are conservative;<br>pass an options object only when the content model intentionally needs extra<br>tags, attributes, or URL schemes.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-search">
  <td><code>search</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#searchoptions">SearchOptions</a> | boolean</code></td>
  <td><div class="ox-api-entry__member-description">Full-text search options.<br>Set to false to disable search.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{ enabled: true }</code></div></td>
</tr>
<tr id="oxcontentoptions-sitemaps">
  <td><code>siteMaps</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#sitemapsoptions">SiteMapsOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Write crawl manifests next to generated HTML.<br><br>Off by default. <code>true</code> writes <code>sitemap.xml</code>, <code>robots.txt</code>, and <code>llms.txt</code>.<br>An object enables the feature and overrides only the fields you set.<br>Requires <code>ssg.siteUrl</code>. When that is missing the build continues and a<br>warning is emitted instead of writing files.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-srcdir">
  <td><code>srcDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Directory containing Markdown source files.<br><br>The path is resolved from the Vite project root. SSG, search indexing, and<br>dev-server routing all use this directory as the content root.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;content&#39;</code></div></td>
</tr>
<tr id="oxcontentoptions-ssg">
  <td><code>ssg</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgoptions">SsgOptions</a> | boolean</code></td>
  <td><div class="ox-api-entry__member-description">Static Site Generation options.<br><br>Passing <code>true</code> or omitting this option enables SSG with defaults. Passing<br><code>false</code> disables the SSG plugin while still allowing Markdown module<br>transforms to run.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{ enabled: true }</code></div></td>
</tr>
<tr id="oxcontentoptions-steps">
  <td><code>steps</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#stepsoptions">StepsOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Restyle a <code>::: steps</code> wrapper around an ordered list.<br><br>Disabled when omitted or <code>false</code>. <code>true</code> and <code>{}</code> enable the default<br>step-list markup. Ordinary ordered lists outside <code>::: steps</code> are unchanged.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-strikethrough">
  <td><code>strikethrough</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable strikethrough.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-tables">
  <td><code>tables</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable tables.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-tasklists">
  <td><code>taskLists</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable task lists.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-toc">
  <td><code>toc</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Generate table of contents.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="oxcontentoptions-tocmaxdepth">
  <td><code>tocMaxDepth</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Maximum heading depth for TOC.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">3</code></div></td>
</tr>
<tr id="oxcontentoptions-transformers">
  <td><code>transformers</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdowntransformer">MarkdownTransformer</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Custom AST transformers.<br>Transformers run after parsing and before the final JavaScript module is emitted.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="oxcontentoptions-wikilinks">
  <td><code>wikiLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#wikilinkoptions">WikiLinkOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Expand Obsidian-style <code>[[page]]</code> and <code>[[page|label]]</code> links.<br><br>Use this for knowledge-base style content where authors prefer short,<br>document-relative link syntax. Pass an object to override the base URL used<br>when resolving generated hrefs.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="paramdoc" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ParamDoc</code><span class="ox-api-entry__description">Parameter documentation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Parameter documentation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ParamDoc</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2631-L2646" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="paramdoc-default">
  <td><code>default</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Default value extracted from syntax or <code>@default</code> tags.</div></td>
</tr>
<tr id="paramdoc-description">
  <td><code>description</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Prose extracted from <code>@param</code> / <code>@arg</code> documentation.</div></td>
</tr>
<tr id="paramdoc-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Parameter name, including dotted names for destructured properties.</div></td>
</tr>
<tr id="paramdoc-optional">
  <td><code>optional</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">True when the parameter is optional.</div></td>
</tr>
<tr id="paramdoc-type">
  <td><code>type</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered TypeScript type text.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="permalinksoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PermalinksOptions</code><span class="ox-api-entry__description">Opt-in frontmatter permalink / slug routing. false or omitted stays off. true o…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in frontmatter <code>permalink</code> / <code>slug</code> routing.</p>
<p><code>false</code> or omitted stays off. <code>true</code> or <code>{}</code> enables defaults. Set <code>enabled: false</code> on the object to turn the feature back off.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface PermalinksOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L647-L653" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="permalinksoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable permalink / slug routing.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="publishstateoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PublishStateOptions</code><span class="ox-api-entry__description">Opt-in draft / unlisted / scheduled page filtering.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in draft / unlisted / scheduled page filtering.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface PublishStateOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L612-L630" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="publishstateoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">When <code>false</code>, frontmatter publish fields are ignored.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true when the option is an object</code></div></td>
</tr>
<tr id="publishstateoptions-includedrafts">
  <td><code>includeDrafts</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Keep draft and not-yet-scheduled pages in output. The dev server sets this.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="publishstateoptions-now">
  <td><code>now</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Injected ISO-8601 clock compared against <code>scheduled</code>, <code>date</code>, and <code>expiry</code>.<br>Invalid values fall back to the system clock.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="readerchromeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ReaderChromeOptions</code><span class="ox-api-entry__description">Per-control flags for ssg.readerChrome. Omitted fields stay on when the feature…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Per-control flags for <code>ssg.readerChrome</code>.</p>
<p>Omitted fields stay on when the feature itself is enabled.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ReaderChromeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L420-L443" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="readerchromeoptions-backtotop">
  <td><code>backToTop</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Back-to-top control that appears after the page is scrolled.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="readerchromeoptions-copy">
  <td><code>copy</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Copy button on fenced code blocks. The clipboard is read in the browser,<br>never at build time.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="readerchromeoptions-externallinks">
  <td><code>externalLinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Icon and <code>rel=&quot;noopener noreferrer&quot;</code> on outbound <code>http(s)</code> links.<br>Relative, hash, and same-document links are left alone.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="redirectsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">RedirectsOptions</code><span class="ox-api-entry__description">Opt-in static redirects, aliases, and path rewrites. A path map such as { &quot;/old…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in static redirects, aliases, and path rewrites.</p>
<p>A path map such as <code>{ &quot;/old-guide&quot;: &quot;/guide&quot; }</code> is also accepted in place of this object and enables the feature with that map.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface RedirectsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L689-L721" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="redirectsoptions-allowexternal">
  <td><code>allowExternal</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Allow <code>http://</code> and <code>https://</code> destinations. <code>javascript:</code>, <code>data:</code>, and<br>protocol-relative <code>//</code> targets stay rejected.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="redirectsoptions-headers">
  <td><code>headers</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Write a <code>_headers</code> Location map next to the HTML pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="redirectsoptions-json">
  <td><code>json</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Write a machine-readable <code>redirects.json</code> map.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="redirectsoptions-map">
  <td><code>map</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Old path to new path. Destinations must be same-origin (<code>/</code> but not <code>//</code>)<br>unless <code>allowExternal</code> is set.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{}</code></div></td>
</tr>
<tr id="redirectsoptions-netlify">
  <td><code>netlify</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Write a Netlify / Cloudflare <code>_redirects</code> file next to the HTML pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolveda11y" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedA11y = false | { skipLinkLabel: string }</code><span class="ox-api-entry__description">Resolved skip-link / print styles. false means no extra markup or CSS.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved skip-link / print styles. <code>false</code> means no extra markup or CSS.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type ResolvedA11y = false | { skipLinkLabel: string }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L473-L477" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="resolvedattrsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedAttrsOptions</code><span class="ox-api-entry__description">Resolved attrs transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved attrs transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedAttrsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1581-L1583" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedattrsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedbadgeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedBadgeOptions</code><span class="ox-api-entry__description">Resolved inline-badge transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved inline-badge transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedBadgeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1421-L1423" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedbadgeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedbuiltinembedoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedBuiltinEmbedOptions</code><span class="ox-api-entry__description">Resolved built-in embed configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">8 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved built-in embed configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedBuiltinEmbedOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1395-L1404" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedbuiltinembedoptions-bluesky">
  <td><code>bluesky</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-github">
  <td><code>github</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">GitHubOptions | false</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-opengraph">
  <td><code>openGraph</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">OgpOptions | false</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-pm">
  <td><code>pm</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#builtinpmoptions">BuiltinPmOptions</a> | false</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-spotify">
  <td><code>spotify</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-stackblitz">
  <td><code>stackBlitz</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-twitter">
  <td><code>twitter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">TwitterEmbedOptions | false</code></td>
  <td></td>
</tr>
<tr id="resolvedbuiltinembedoptions-webcontainer">
  <td><code>webContainer</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcardoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCardOptions</code><span class="ox-api-entry__description">Resolved card transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved card transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCardOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1656-L1658" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcardoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcascadeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCascadeOptions</code><span class="ox-api-entry__description">Resolved cascade options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved cascade options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCascadeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L679-L681" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcascadeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcodeannotationsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCodeAnnotationsOptions</code><span class="ox-api-entry__description">Resolved code annotation configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved code annotation configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCodeAnnotationsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2002-L2007" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcodeannotationsoptions-defaultlinenumbers">
  <td><code>defaultLineNumbers</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeannotationsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeannotationsoptions-metakey">
  <td><code>metaKey</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeannotationsoptions-notation">
  <td><code>notation</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#codeannotationsyntax">CodeAnnotationSyntax</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcodeblocklintoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCodeBlockLintOptions</code><span class="ox-api-entry__description">Resolved code-block lint options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved code-block lint options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCodeBlockLintOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1853-L1859" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcodeblocklintoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocklintoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocklintoptions-mode">
  <td><code>mode</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;warn&quot; | &quot;error&quot;</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocklintoptions-requirelanguage">
  <td><code>requireLanguage</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocklintoptions-trailingspaces">
  <td><code>trailingSpaces</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcodeblocktypecheckoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCodeBlockTypecheckOptions</code><span class="ox-api-entry__description">Resolved code-block type-check options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved code-block type-check options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCodeBlockTypecheckOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1910-L1916" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcodeblocktypecheckoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocktypecheckoptions-languages">
  <td><code>languages</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocktypecheckoptions-mode">
  <td><code>mode</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;warn&quot; | &quot;error&quot;</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocktypecheckoptions-requiremeta">
  <td><code>requireMeta</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeblocktypecheckoptions-tsgocommand">
  <td><code>tsgoCommand</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcodeimportoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCodeImportOptions</code><span class="ox-api-entry__description">Resolved code-import transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved code-import transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCodeImportOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1611-L1614" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcodeimportoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcodeimportoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcollectionoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCollectionOptions</code><span class="ox-api-entry__description">Resolved collection definition.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved collection definition.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCollectionOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2803-L2807" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcollectionoptions-include">
  <td><code>include</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#collectionincludefield">CollectionIncludeField</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolvedcollectionoptions-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedcollectionoptions-source">
  <td><code>source</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcollectionsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedCollectionsOptions</code><span class="ox-api-entry__description">Resolved collection options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved collection options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedCollectionsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2812-L2815" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcollectionsoptions-collections">
  <td><code>collections</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, <a href="#resolvedcollectionoptions">ResolvedCollectionOptions</a>&gt;</code></td>
  <td></td>
</tr>
<tr id="resolvedcollectionsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedcontaineroptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedContainerOptions</code><span class="ox-api-entry__description">Resolved custom-container transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved custom-container transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedContainerOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1457-L1460" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedcontaineroptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedcontaineroptions-types">
  <td><code>types</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, <a href="#containertypeoptions">ContainerTypeOptions</a>&gt;</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolveddocsentrypoint" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedDocsEntryPoint</code><span class="ox-api-entry__description">Resolved public API entry point.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved public API entry point.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedDocsEntryPoint</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2204-L2207" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolveddocsentrypoint-name">
  <td><code>name</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolveddocsentrypoint-path">
  <td><code>path</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolveddocsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedDocsOptions</code><span class="ox-api-entry__description">Resolved docs options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">33 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved docs options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedDocsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2488-L2522" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolveddocsoptions-basepath">
  <td><code>basePath</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-classpropertiesformat">
  <td><code>classPropertiesFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-entrypoints">
  <td><code>entryPoints</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolveddocsentrypoint">ResolvedDocsEntryPoint</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-enummembersformat">
  <td><code>enumMembersFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-exclude">
  <td><code>exclude</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-format">
  <td><code>format</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;markdown&quot; | &quot;json&quot; | &quot;html&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-generatenav">
  <td><code>generateNav</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-githuburl">
  <td><code>githubUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-groupby">
  <td><code>groupBy</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;file&quot; | &quot;category&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-grouporder">
  <td><code>groupOrder</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-include">
  <td><code>include</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-indexformat">
  <td><code>indexFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-interfacepropertiesformat">
  <td><code>interfacePropertiesFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-internal">
  <td><code>internal</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-kindsortorder">
  <td><code>kindSortOrder</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-linkstyle">
  <td><code>linkStyle</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;markdown&quot; | &quot;clean&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-out">
  <td><code>out</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-parametersformat">
  <td><code>parametersFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-pathstrategy">
  <td><code>pathStrategy</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;flat&quot; | &quot;typedoc&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-private">
  <td><code>private</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-propertymembersformat">
  <td><code>propertyMembersFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-rendergeneratedby">
  <td><code>renderGeneratedBy</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-renderstats">
  <td><code>renderStats</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-renderstyle">
  <td><code>renderStyle</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;html&quot; | &quot;markdown&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-singleentryroot">
  <td><code>singleEntryRoot</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;preserve&quot; | &quot;flatten&quot;</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-sort">
  <td><code>sort</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">DocsSortStrategy[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-sortentrypoints">
  <td><code>sortEntryPoints</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-src">
  <td><code>src</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-toc">
  <td><code>toc</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-typealiaspropertiesformat">
  <td><code>typeAliasPropertiesFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-typedeclarationformat">
  <td><code>typeDeclarationFormat</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownDisplayFormat</code></td>
  <td></td>
</tr>
<tr id="resolveddocsoptions-typeparameters">
  <td><code>typeParameters</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolveddocstestoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedDocsTestOptions</code><span class="ox-api-entry__description">Resolved docs-as-tests extraction options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved docs-as-tests extraction options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedDocsTestOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1948-L1952" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolveddocstestoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolveddocstestoptions-languages">
  <td><code>languages</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolveddocstestoptions-requiremeta">
  <td><code>requireMeta</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvededitthispageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedEditThisPageOptions</code><span class="ox-api-entry__description">Resolved edit-link transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved edit-link transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedEditThisPageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1796-L1802" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvededitthispageoptions-branch">
  <td><code>branch</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvededitthispageoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvededitthispageoptions-label">
  <td><code>label</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvededitthispageoptions-repourl">
  <td><code>repoUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvededitthispageoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedemojishortcodeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedEmojiShortcodeOptions</code><span class="ox-api-entry__description">Resolved emoji-shortcode transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved emoji-shortcode transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedEmojiShortcodeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1535-L1538" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedemojishortcodeoptions-custom">
  <td><code>custom</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td></td>
</tr>
<tr id="resolvedemojishortcodeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedfeedsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedFeedsOptions</code><span class="ox-api-entry__description">Resolved feed options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved feed options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedFeedsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L772-L778" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedfeedsoptions-collection">
  <td><code>collection</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedfeedsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedfeedsoptions-formats">
  <td><code>formats</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#feedformat">FeedFormat</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolvedfeedsoptions-limit">
  <td><code>limit</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedfeedsoptions-path">
  <td><code>path</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedfiletreeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedFileTreeOptions</code><span class="ox-api-entry__description">Resolved file-tree transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved file-tree transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedFileTreeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1694-L1696" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedfiletreeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedi18noptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedI18nOptions</code><span class="ox-api-entry__description">Resolved i18n options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved i18n options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedI18nOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L3113-L3121" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedi18noptions-check">
  <td><code>check</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-defaultlocale">
  <td><code>defaultLocale</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-dir">
  <td><code>dir</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-functionnames">
  <td><code>functionNames</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-hidedefaultlocale">
  <td><code>hideDefaultLocale</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedi18noptions-locales">
  <td><code>locales</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#localeconfig">LocaleConfig</a>[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedImageOptions</code><span class="ox-api-entry__description">Resolved image transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved image transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1477-L1480" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedimageoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedimageoptions-lazy">
  <td><code>lazy</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedincludeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedIncludeOptions</code><span class="ox-api-entry__description">Resolved Markdown-include transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved Markdown-include transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedIncludeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1636-L1639" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedincludeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedincludeoptions-rootdir">
  <td><code>rootDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedmathoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedMathOptions</code><span class="ox-api-entry__description">Resolved math transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved math transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedMathOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1555-L1557" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedmathoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvednotfoundoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedNotFoundOptions</code><span class="ox-api-entry__description">Resolved custom 404 options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved custom 404 options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedNotFoundOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L534-L538" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvednotfoundoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvednotfoundoptions-output">
  <td><code>output</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvednotfoundoptions-source">
  <td><code>source</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedogimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedOgImageOptions</code><span class="ox-api-entry__description">Resolved OG image options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved OG image options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedOgImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2061-L2068" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedogimageoptions-cache">
  <td><code>cache</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-concurrency">
  <td><code>concurrency</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-height">
  <td><code>height</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-template">
  <td><code>template</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-vueplugin">
  <td><code>vuePlugin</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;vitejs&quot; | &quot;vizejs&quot;</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-width">
  <td><code>width</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedOptions</code><span class="ox-api-entry__description">Resolved options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">50 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1266-L1317" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedoptions-attrs">
  <td><code>attrs</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedattrsoptions">ResolvedAttrsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-autolinks">
  <td><code>autolinks</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-badges">
  <td><code>badges</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedbadgeoptions">ResolvedBadgeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-base">
  <td><code>base</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-cards">
  <td><code>cards</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcardoptions">ResolvedCardOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-cascade">
  <td><code>cascade</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcascadeoptions">ResolvedCascadeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-cjkemphasis">
  <td><code>cjkEmphasis</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-codeannotations">
  <td><code>codeAnnotations</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcodeannotationsoptions">ResolvedCodeAnnotationsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-codeblocklint">
  <td><code>codeBlockLint</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcodeblocklintoptions">ResolvedCodeBlockLintOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-codeblocktypecheck">
  <td><code>codeBlockTypecheck</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcodeblocktypecheckoptions">ResolvedCodeBlockTypecheckOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-codeimports">
  <td><code>codeImports</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcodeimportoptions">ResolvedCodeImportOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-collections">
  <td><code>collections</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcollectionsoptions">ResolvedCollectionsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-containers">
  <td><code>containers</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedcontaineroptions">ResolvedContainerOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-docs">
  <td><code>docs</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolveddocsoptions">ResolvedDocsOptions</a> | false</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-docstests">
  <td><code>docsTests</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolveddocstestoptions">ResolvedDocsTestOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-editthispage">
  <td><code>editThisPage</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvededitthispageoptions">ResolvedEditThisPageOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-embeds">
  <td><code>embeds</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedbuiltinembedoptions">ResolvedBuiltinEmbedOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-emojishortcodes">
  <td><code>emojiShortcodes</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedemojishortcodeoptions">ResolvedEmojiShortcodeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-extensions">
  <td><code>extensions</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-feeds">
  <td><code>feeds</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedfeedsoptions">ResolvedFeedsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-filetree">
  <td><code>fileTree</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedfiletreeoptions">ResolvedFileTreeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-footnotes">
  <td><code>footnotes</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-gfm">
  <td><code>gfm</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-highlight">
  <td><code>highlight</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-i18n">
  <td><code>i18n</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedi18noptions">ResolvedI18nOptions</a> | false</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-images">
  <td><code>images</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedimageoptions">ResolvedImageOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-includes">
  <td><code>includes</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedincludeoptions">ResolvedIncludeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-math">
  <td><code>math</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedmathoptions">ResolvedMathOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-mermaid">
  <td><code>mermaid</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-ogimage">
  <td><code>ogImage</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-ogimageoptions">
  <td><code>ogImageOptions</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedogimageoptions">ResolvedOgImageOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-ogviewer">
  <td><code>ogViewer</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-outdir">
  <td><code>outDir</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-permalinks">
  <td><code>permalinks</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedpermalinksoptions">ResolvedPermalinksOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-publishstate">
  <td><code>publishState</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedpublishstateoptions">ResolvedPublishStateOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-redirects">
  <td><code>redirects</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedredirectsoptions">ResolvedRedirectsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-sanitize">
  <td><code>sanitize</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedsanitizeoptions">ResolvedSanitizeOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-search">
  <td><code>search</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedsearchoptions">ResolvedSearchOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-sitemaps">
  <td><code>siteMaps</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedsitemapsoptions">ResolvedSiteMapsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-srcdir">
  <td><code>srcDir</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-ssg">
  <td><code>ssg</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedssgoptions">ResolvedSsgOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-steps">
  <td><code>steps</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedstepsoptions">ResolvedStepsOptions</a></code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-strikethrough">
  <td><code>strikethrough</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-tables">
  <td><code>tables</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-tasklists">
  <td><code>taskLists</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-toc">
  <td><code>toc</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-tocmaxdepth">
  <td><code>tocMaxDepth</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-transformers">
  <td><code>transformers</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdowntransformer">MarkdownTransformer</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-wikilinks">
  <td><code>wikiLinks</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedwikilinkoptions">ResolvedWikiLinkOptions</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedpermalinksoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedPermalinksOptions</code><span class="ox-api-entry__description">Resolved permalink options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved permalink options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedPermalinksOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L658-L660" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedpermalinksoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedpublishstateoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedPublishStateOptions</code><span class="ox-api-entry__description">Resolved publish-state options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved publish-state options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedPublishStateOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L635-L639" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedpublishstateoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedpublishstateoptions-includedrafts">
  <td><code>includeDrafts</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedpublishstateoptions-now">
  <td><code>now</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedreaderchrome" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedReaderChrome = false | { copy: boolean; externalLinks: boolean; backToTop: boolean }</code><span class="ox-api-entry__description">Resolved reader chrome. false means no extra markup or JS.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved reader chrome. <code>false</code> means no extra markup or JS.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type ResolvedReaderChrome = false | { copy: boolean; externalLinks: boolean; backToTop: boolean }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L448-L454" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="resolvedredirectsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedRedirectsOptions</code><span class="ox-api-entry__description">Resolved redirect options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved redirect options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedRedirectsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L726-L733" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedredirectsoptions-allowexternal">
  <td><code>allowExternal</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedredirectsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedredirectsoptions-headers">
  <td><code>headers</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedredirectsoptions-json">
  <td><code>json</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedredirectsoptions-map">
  <td><code>map</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, string&gt;</code></td>
  <td></td>
</tr>
<tr id="resolvedredirectsoptions-netlify">
  <td><code>netlify</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedsanitizeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedSanitizeOptions</code><span class="ox-api-entry__description">Resolved sanitize transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved sanitize transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedSanitizeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1737-L1742" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedsanitizeoptions-allowedattributes">
  <td><code>allowedAttributes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedsanitizeoptions-allowedtags">
  <td><code>allowedTags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedsanitizeoptions-allowedurlschemes">
  <td><code>allowedUrlSchemes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="resolvedsanitizeoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedsearchoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedSearchOptions</code><span class="ox-api-entry__description">Resolved search options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved search options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedSearchOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2941-L2947" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedsearchoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedsearchoptions-hotkey">
  <td><code>hotkey</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedsearchoptions-limit">
  <td><code>limit</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedsearchoptions-placeholder">
  <td><code>placeholder</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedsearchoptions-prefix">
  <td><code>prefix</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedsitemapsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedSiteMapsOptions</code><span class="ox-api-entry__description">Resolved crawl-manifest options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved crawl-manifest options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedSiteMapsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L603-L607" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedsitemapsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedsitemapsoptions-llms">
  <td><code>llms</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedsitemapsoptions-robots">
  <td><code>robots</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedssgoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedSsgOptions</code><span class="ox-api-entry__description">Resolved SSG options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">23 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved SSG options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedSsgOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L482-L512" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedssgoptions-a11y">
  <td><code>a11y</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolveda11y">ResolvedA11y</a></code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-bare">
  <td><code>bare</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-bodyend">
  <td><code>bodyEnd</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-bodystart">
  <td><code>bodyStart</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-breadcrumbs">
  <td><code>breadcrumbs</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-clean">
  <td><code>clean</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-extension">
  <td><code>extension</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-generateogimage">
  <td><code>generateOgImage</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-head">
  <td><code>head</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-lang">
  <td><code>lang</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-lastupdated">
  <td><code>lastUpdated</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-localeswitcher">
  <td><code>localeSwitcher</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-navigation">
  <td><code>navigation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgnavigationgroup">SsgNavigationGroup</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-notfound">
  <td><code>notFound</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvednotfoundoptions">ResolvedNotFoundOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Present after <code>resolveSsgOptions</code>. Omitted in hand-built fixtures means off.</div></td>
</tr>
<tr id="resolvedssgoptions-ogimage">
  <td><code>ogImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-pagination">
  <td><code>pagination</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-readerchrome">
  <td><code>readerChrome</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedreaderchrome">ResolvedReaderChrome</a></code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-render">
  <td><code>render</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-renderer.md#themecomponent">ThemeComponent</a></code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-sitename">
  <td><code>siteName</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-siteurl">
  <td><code>siteUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-team">
  <td><code>team</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedteamoptions">ResolvedTeamOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Present after <code>resolveSsgOptions</code>. Omitted in hand-built fixtures means off.</div></td>
</tr>
<tr id="resolvedssgoptions-theme">
  <td><code>theme</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme.md#resolvedthemeconfig">ResolvedThemeConfig</a></code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedstepsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedStepsOptions</code><span class="ox-api-entry__description">Resolved step-list transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved step-list transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedStepsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1675-L1677" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedstepsoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedteamoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedTeamOptions</code><span class="ox-api-entry__description">Resolved team page options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved team page options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedTeamOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L578-L581" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedteamoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedteamoptions-members">
  <td><code>members</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#teammember">TeamMember</a>[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedwikilinkoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedWikiLinkOptions</code><span class="ox-api-entry__description">Resolved wiki-link transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved wiki-link transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedWikiLinkOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1504-L1507" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedwikilinkoptions-baseurl">
  <td><code>baseUrl</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedwikilinkoptions-enabled">
  <td><code>enabled</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="returndoc" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ReturnDoc</code><span class="ox-api-entry__description">Return type documentation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Return type documentation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ReturnDoc</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2651-L2657" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="returndoc-description">
  <td><code>description</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Prose extracted from <code>@returns</code> / <code>@return</code> documentation.</div></td>
</tr>
<tr id="returndoc-type">
  <td><code>type</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered TypeScript type text for the return value.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="sanitizeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SanitizeOptions</code><span class="ox-api-entry__description">Options for sanitizing rendered HTML. Sanitization happens after Markdown is re…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for sanitizing rendered HTML.</p>
<p>Sanitization happens after Markdown is rendered to HTML. This makes it useful for user-authored content, but consumers should avoid enabling extra tags or schemes unless the rendered output explicitly requires them.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SanitizeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1705-L1732" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="sanitizeoptions-allowedattributes">
  <td><code>allowedAttributes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Allowed HTML attribute names. Omit to use the built-in safe attribute allow list.<br><br>Provide a full replacement list, not a list of additions.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="sanitizeoptions-allowedtags">
  <td><code>allowedTags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Allowed HTML tag names. Omit to use the built-in safe tag allow list.<br><br>Provide a full replacement list, not a list of additions.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="sanitizeoptions-allowedurlschemes">
  <td><code>allowedUrlSchemes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Allowed URL schemes for link-like attributes.<br><br>Omit to use the built-in safe scheme allow list.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="scopedsearchquery" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ScopedSearchQuery</code><span class="ox-api-entry__description">Parsed search query with optional scope prefixes.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Parsed search query with optional scope prefixes.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ScopedSearchQuery</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L3001-L3007" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="scopedsearchquery-scopes">
  <td><code>scopes</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Deduplicated lowercase scope prefixes requested by the query.</div></td>
</tr>
<tr id="scopedsearchquery-text">
  <td><code>text</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Query text after <code>@scope</code> prefixes have been removed.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="searchdocument" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SearchDocument</code><span class="ox-api-entry__description">Search document structure.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Search document structure.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SearchDocument</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2952-L2970" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="searchdocument-body">
  <td><code>body</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Plain-text body content used for scoring and snippets.</div></td>
</tr>
<tr id="searchdocument-code">
  <td><code>code</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Code block text extracted from the document.</div></td>
</tr>
<tr id="searchdocument-headings">
  <td><code>headings</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Headings extracted from the document.</div></td>
</tr>
<tr id="searchdocument-id">
  <td><code>id</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Stable document identifier used by the search index.</div></td>
</tr>
<tr id="searchdocument-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Human-readable document title.</div></td>
</tr>
<tr id="searchdocument-url">
  <td><code>url</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">URL returned to search consumers.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="searchoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SearchOptions</code><span class="ox-api-entry__description">Options for full-text search. Search indexes are built from Markdown content at…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for full-text search.</p>
<p>Search indexes are built from Markdown content at build time and loaded by the client runtime from <code>search-index.json</code>. Pass <code>false</code> to the top-level <code>search</code> option to disable both index generation and the virtual search module.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SearchOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2888-L2936" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="searchoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable search functionality.<br><br>Set this to <code>false</code> when config merging requires an object shape but search<br>should be disabled.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="searchoptions-hotkey">
  <td><code>hotkey</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Keyboard shortcut to focus search (without modifier).<br><br>Use an empty string to let the UI opt out of registering a shortcut.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;/&#39;</code></div></td>
</tr>
<tr id="searchoptions-limit">
  <td><code>limit</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Maximum number of search results.<br><br>This controls client-side result truncation, not the number of documents in<br>the generated index.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">10</code></div></td>
</tr>
<tr id="searchoptions-placeholder">
  <td><code>placeholder</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Placeholder text for the search input.<br><br>This value is embedded in the virtual search module for UI consumers.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;Search documentation...&#39;</code></div></td>
</tr>
<tr id="searchoptions-prefix">
  <td><code>prefix</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable prefix matching for autocomplete.<br><br>Prefix matching applies to the final query token, which keeps normal terms<br>precise while still supporting typeahead-style interactions.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="searchresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SearchResult</code><span class="ox-api-entry__description">Search result structure.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Search result structure.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SearchResult</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2975-L2996" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="searchresult-id">
  <td><code>id</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Matching document identifier.</div></td>
</tr>
<tr id="searchresult-matches">
  <td><code>matches</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Query terms that matched the document.</div></td>
</tr>
<tr id="searchresult-scopes">
  <td><code>scopes</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Hierarchical scopes derived from the result URL or document id.</div></td>
</tr>
<tr id="searchresult-score">
  <td><code>score</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Relevance score returned by the BM25 search engine.</div></td>
</tr>
<tr id="searchresult-snippet">
  <td><code>snippet</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Context snippet with highlighted terms when available.</div></td>
</tr>
<tr id="searchresult-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Matching document title.</div></td>
</tr>
<tr id="searchresult-url">
  <td><code>url</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">URL to open when the result is selected.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="sitemapsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SiteMapsOptions</code><span class="ox-api-entry__description">Opt-in crawl manifests written during SSG.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in crawl manifests written during SSG.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SiteMapsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L586-L598" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="sitemapsoptions-llms">
  <td><code>llms</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Write <code>llms.txt</code> with the site title, description, and page URLs.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="sitemapsoptions-robots">
  <td><code>robots</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Write <code>robots.txt</code> with a Sitemap line.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ssgnavigationgroup" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgNavigationGroup</code><span class="ox-api-entry__description">Navigation group for SSG sidebar rendering.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Navigation group for SSG sidebar rendering.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgNavigationGroup</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L141-L147" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgnavigationgroup-items">
  <td><code>items</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgnavigationitem">SsgNavigationItem</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Navigation items within this group</div></td>
</tr>
<tr id="ssgnavigationgroup-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Group heading</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ssgnavigationitem" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgNavigationItem</code><span class="ox-api-entry__description">Navigation item for SSG sidebar rendering.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Navigation item for SSG sidebar rendering.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgNavigationItem</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L121-L136" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgnavigationitem-href">
  <td><code>href</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Final href used in the rendered HTML.<br>When omitted for internal links, ox-content derives it from <code>path</code>.</div></td>
</tr>
<tr id="ssgnavigationitem-path">
  <td><code>path</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Route path used for active-state matching.<br>Internal links should use site-relative paths such as <code>/getting-started</code>.</div></td>
</tr>
<tr id="ssgnavigationitem-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Display title</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ssgoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgOptions</code><span class="ox-api-entry__description">Static Site Generation options. These options control the HTML files emitted at…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">23 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Static Site Generation options.</p>
<p>These options control the HTML files emitted at build time and the matching dev-server preview behavior. Pass <code>false</code> to the top-level <code>ssg</code> option to disable the whole SSG pipeline, or pass an object to customize the defaults.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L156-L413" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgoptions-a11y">
  <td><code>a11y</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#a11yoptions">A11yOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in skip link and print styles.<br><br>Disabled when omitted or <code>false</code>. <code>true</code> enables the default skip link<br>and print CSS. An object enables the feature and can override the label.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-bare">
  <td><code>bare</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Emit bare HTML with only the rendered Markdown body.<br><br>This skips the default navigation, layout shell, and theme styles. It is<br>mainly useful for benchmarking, fixture generation, or projects that wrap<br>the output in their own shell.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-bodyend">
  <td><code>bodyEnd</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Raw markup inserted directly before <code>&lt;/body&gt;</code>.<br><br>Bare mode only. Use it for a site footer, or scripts you inject yourself.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-bodystart">
  <td><code>bodyStart</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Raw markup inserted directly after <code>&lt;body&gt;</code>.<br><br>Bare mode only. Use it for a site header that wraps the rendered page.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-breadcrumbs">
  <td><code>breadcrumbs</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Show a breadcrumb trail from the site root through sidebar ancestors.<br><br>Disabled when omitted or <code>false</code>. <code>true</code> enables the default trail.<br>An object also enables the feature. Frontmatter <code>breadcrumbs: false</code><br>hides the trail on that page.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-clean">
  <td><code>clean</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Remove previously generated files from the output directory before writing<br>the new SSG result.<br><br>Leave this disabled when the output directory also contains assets produced<br>by other Vite plugins or external build steps.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the SSG pipeline.<br><br>Keep this enabled when ox-content owns page rendering. Disable it only when<br>another framework integration will consume the Markdown modules directly.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="ssgoptions-extension">
  <td><code>extension</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">File extension used for generated routes.<br><br>The value should include the leading dot. For example, <code>.html</code> emits<br><code>guide.html</code>, while an empty string can be used by custom deployments that<br>map extensionless output themselves.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;.html&#39;</code></div></td>
</tr>
<tr id="ssgoptions-generateogimage">
  <td><code>generateOgImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Generate one Open Graph image per page.<br><br>Generated images are written alongside the SSG output and referenced from<br>each page&#39;s metadata. Configure rendering details with the top-level<br><code>ogImageOptions</code> option.<br><br>Under <code>bare</code>, the images are still written but nothing references them,<br>because bare output has no <code>&lt;head&gt;</code> to put the <code>&lt;meta&gt;</code> tags in — inject<br>them from your own shell.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-head">
  <td><code>head</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Raw markup appended to <code>&lt;head&gt;</code>.<br><br>Bare mode only — themed pages own their head. Use it for the stylesheet<br>your own build emits, or any tag the plugin does not generate.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-lang">
  <td><code>lang</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description"><code>lang</code> attribute for the generated <code>&lt;html&gt;</code> element.<br><br>Bare mode uses this verbatim; themed pages derive it from <code>i18n</code> instead.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;en&quot;</code></div></td>
</tr>
<tr id="ssgoptions-lastupdated">
  <td><code>lastUpdated</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Add each page&#39;s last git commit timestamp to the default theme.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-localeswitcher">
  <td><code>localeSwitcher</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Show a header locale switcher in the default theme.<br><br>Disabled when omitted or <code>false</code>, even if <code>i18n.locales</code> is set.<br><code>true</code> or an object enables the control when available locales are<br>non-empty. Links use the sibling page when it exists, otherwise the<br>locale root (<code>/{locale}/</code> or a configured root).</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-navigation">
  <td><code>navigation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgnavigationgroup">SsgNavigationGroup</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Sidebar navigation override.<br><br>When omitted, ox-content derives navigation from the Markdown file tree.<br>Provide this when migrating from systems such as VitePress where navigation<br>is intentionally hand-authored.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-notfound">
  <td><code>notFound</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#notfoundoptions">NotFoundOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Write a themed 404 page during SSG.<br><br>Off by default. <code>true</code> reads <code>404.md</code> from <code>srcDir</code> and writes <code>404.html</code>.<br>An object enables the feature and overrides only the fields you set.<br>When the source file is missing, a built-in &quot;Page not found&quot; page is<br>written instead. The page is omitted from the search index and sitemap.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-ogimage">
  <td><code>ogImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Static Open Graph image URL used for social sharing.<br><br>When <code>generateOgImage</code> is enabled, this value is still useful as a fallback<br>for pages that cannot produce a generated image.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-pagination">
  <td><code>pagination</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Show previous/next page links after the article.<br><br>Disabled when omitted or <code>false</code>. <code>true</code> enables the default pager.<br>An object also enables the feature.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-readerchrome">
  <td><code>readerChrome</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#readerchromeoptions">ReaderChromeOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Opt-in copy buttons, outbound-link icons, and a back-to-top control.<br><br>Disabled when omitted or <code>false</code>. <code>true</code> enables all three with defaults.<br>An object enables the feature and can turn one control off, for example<br><code>{ copy: false }</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-render">
  <td><code>render</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme-renderer.md#themecomponent">ThemeComponent</a></code></td>
  <td><div class="ox-api-entry__member-description">Render each page with a JSX theme component instead of the built-in<br>renderer.<br><br>The component owns the whole document, so <code>theme</code>, <code>bare</code> and the head<br>metadata options do not apply — everything from <code>&lt;html&gt;</code> down is yours.<br>Compose one per layout with <code>createTheme()</code>, and read the current page<br>through <code>usePageProps()</code> / <code>useSiteConfig()</code>.<br><br>``<code>ts<br>ssg: { render: createTheme({ layouts: { default: DefaultLayout } }) }<br></code>``</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-sitename">
  <td><code>siteName</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site name shown in the default theme header and title suffix.<br><br>When omitted, the renderer falls back to project metadata where available.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-siteurl">
  <td><code>siteUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Absolute site URL used when generating social metadata.<br><br>Set this when pages need absolute Open Graph image URLs. Include the origin<br>and any deployment base path, without a trailing page path.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-team">
  <td><code>team</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean | <a href="#teamoptions">TeamOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Render a static members card grid on pages with <code>layout: team</code>.<br><br>Off by default. <code>true</code> enables an empty list. An object enables the<br>feature and supplies <code>members</code>. When the option is off, <code>layout: team</code><br>is ignored and the page stays ordinary.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-theme">
  <td><code>theme</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme.md#themeconfig">ThemeConfig</a> | <a href="./theme.md#themeconfig">ThemeConfig</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Theme configuration for generated pages.<br><br>Use <code>defineTheme()</code> to build this object so custom theme modules and the<br>default theme extension points keep their expected shape.<br><br>An array composes layers left to right, which is how a skin package and a<br>color package are combined:<br><br>``<code>ts<br>theme: [pixelSkin, tokyoNight, { footer: { copyright: &quot;2026&quot; } }]<br></code>``</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">defaultTheme</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="stepsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">StepsOptions</code><span class="ox-api-entry__description">Options for opt-in ::: steps ordered lists.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for opt-in <code>::: steps</code> ordered lists.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface StepsOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1663-L1670" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="stepsoptions-enabled">
  <td><code>enabled</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable the steps transform when an options object is supplied.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="teamlink" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TeamLink</code><span class="ox-api-entry__description">One link on a team member card.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>One link on a team member card.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TeamLink</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L543-L548" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="teamlink-href">
  <td><code>href</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Destination. Only <code>https:</code> or a site-relative <code>/</code> path is emitted.</div></td>
</tr>
<tr id="teamlink-label">
  <td><code>label</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Visible label. Escaped in HTML.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="teammember" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TeamMember</code><span class="ox-api-entry__description">One person on the team page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>One person on the team page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TeamMember</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L553-L562" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="teammember-avatar">
  <td><code>avatar</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Avatar URL. Only <code>https:</code> or a site-relative <code>/</code> path is emitted.</div></td>
</tr>
<tr id="teammember-links">
  <td><code>links</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#teamlink">TeamLink</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Optional profile or social links.</div></td>
</tr>
<tr id="teammember-name">
  <td><code>name</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Display name. Escaped in HTML.</div></td>
</tr>
<tr id="teammember-role">
  <td><code>role</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Optional role or title. Escaped in HTML.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="teamoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TeamOptions</code><span class="ox-api-entry__description">Opt-in team / members page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in team / members page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TeamOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L567-L573" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="teamoptions-members">
  <td><code>members</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#teammember">TeamMember</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">People rendered as static cards on <code>layout: team</code> pages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="throwsdoc" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThrowsDoc</code><span class="ox-api-entry__description">Exception/error documentation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Exception/error documentation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThrowsDoc</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2662-L2668" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="throwsdoc-description">
  <td><code>description</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Prose extracted from <code>@throws</code> / <code>@exception</code> documentation.</div></td>
</tr>
<tr id="throwsdoc-type">
  <td><code>type</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered TypeScript type text for the thrown value, when documented.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="tocentry" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TocEntry</code><span class="ox-api-entry__description">Table of contents entry.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Table of contents entry.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TocEntry</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2148-L2168" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="tocentry-children">
  <td><code>children</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#tocentry">TocEntry</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Child entries.</div></td>
</tr>
<tr id="tocentry-depth">
  <td><code>depth</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Heading depth (1-6).</div></td>
</tr>
<tr id="tocentry-slug">
  <td><code>slug</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Slug/ID for linking.</div></td>
</tr>
<tr id="tocentry-text">
  <td><code>text</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Heading text.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="transformcontext" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TransformContext</code><span class="ox-api-entry__description">Transform context passed to transformers.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform context passed to transformers.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TransformContext</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2088-L2103" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="transformcontext-filepath">
  <td><code>filePath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">File path being processed.</div></td>
</tr>
<tr id="transformcontext-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Frontmatter data.</div></td>
</tr>
<tr id="transformcontext-options">
  <td><code>options</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedoptions">ResolvedOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Resolved plugin options.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="transformresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">TransformResult</code><span class="ox-api-entry__description">Transform result.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform result.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface TransformResult</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2118-L2143" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="transformresult-code">
  <td><code>code</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Generated JavaScript code.</div></td>
</tr>
<tr id="transformresult-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Parsed frontmatter.</div></td>
</tr>
<tr id="transformresult-html">
  <td><code>html</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered HTML.</div></td>
</tr>
<tr id="transformresult-map">
  <td><code>map</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">null</code></td>
  <td><div class="ox-api-entry__member-description">Source map (null means no source map).</div></td>
</tr>
<tr id="transformresult-toc">
  <td><code>toc</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#tocentry">TocEntry</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Table of contents.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="types" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">types</code><span class="ox-api-entry__description">Type definitions for @ox-content/vite-plugin</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Type definitions for @ox-content/vite-plugin</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="wikilinkoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">WikiLinkOptions</code><span class="ox-api-entry__description">Options for expanding Obsidian-style wiki links. The transform accepts [target]…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for expanding Obsidian-style wiki links.</p>
<p>The transform accepts <code>[[target]]</code> and <code>[[target|label]]</code> syntax and rewrites it to regular links before rendering. It is intentionally small: path resolution is based on the configured base URL rather than a full backlink graph.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface WikiLinkOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1490-L1499" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="wikilinkoptions-baseurl">
  <td><code>baseUrl</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base URL prepended to resolved wiki-link targets.<br><br>When omitted, the top-level <code>base</code> option is used.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">options.base</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

