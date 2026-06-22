# types.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts)**

> 66 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>66</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>62</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>397</strong>
  <span>members</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L796-L807" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L650-L705" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Render <code>&lt;Tweet&gt;</code> / <code>&lt;XPost&gt;</code> as static privacy-conscious cards.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L710-L716" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1106" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1116-L1146" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1111" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L960-L997" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1017-L1054" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L823-L837" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1680-L1722" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1727-L1775" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1326-L1331" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1366-L1632" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1845-L1866" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1073-L1092" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L900-L940" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L766-L780" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1822-L1840" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1871-L1883" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2069-L2137" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2047-L2060" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1257-L1262" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1222-L1232" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1888-L1903" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1162-L1205" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OxContentOptions</code><span class="ox-api-entry__description">Options for the core oxContent() Vite plugin. The top-level options describe wh…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">36 members</span></span></span></summary>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L301-L603" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<tr id="oxcontentoptions-base">
  <td><code>base</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base path prepended to generated internal URLs.<br><br>Use this when the site is deployed below a sub-path, such as GitHub Pages or<br>a documentation route inside a larger application.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;/&#39;</code></div></td>
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
  <td><div class="ox-api-entry__member-description">Enable syntax highlighting for code blocks.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="oxcontentoptions-highlightlangs">
  <td><code>highlightLangs</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">LanguageRegistration[]</code></td>
  <td><div class="ox-api-entry__member-description">Additional languages for syntax highlighting.<br>Accepts Shiki LanguageRegistration objects (e.g., TextMate grammars).<br>These are loaded alongside the built-in languages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="oxcontentoptions-highlighttheme">
  <td><code>highlightTheme</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string | ThemeRegistration</code></td>
  <td><div class="ox-api-entry__member-description">Syntax highlighting theme.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;github-dark&#39;</code></div></td>
</tr>
<tr id="oxcontentoptions-i18n">
  <td><code>i18n</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#i18noptions">I18nOptions</a> | false</code></td>
  <td><div class="ox-api-entry__member-description">i18n (internationalization) options.<br>Set to false to disable i18n.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1780-L1795" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L812-L814" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L721-L730" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1151-L1156" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1002-L1008" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1059-L1065" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L842-L845" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1353-L1356" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1637-L1671" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1097-L1101" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L945-L951" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L785-L788" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2142-L2150" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1210-L1217" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedOptions</code><span class="ox-api-entry__description">Resolved options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">36 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L608-L645" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<tr id="resolvedoptions-base">
  <td><code>base</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
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
<tr id="resolvedoptions-highlightlangs">
  <td><code>highlightLangs</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">LanguageRegistration[]</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-highlighttheme">
  <td><code>highlightTheme</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string | ThemeRegistration</code></td>
  <td></td>
</tr>
<tr id="resolvedoptions-i18n">
  <td><code>i18n</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedi18noptions">ResolvedI18nOptions</a> | false</code></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L886-L891" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1970-L1976" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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

<details id="resolvedssgoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedSsgOptions</code><span class="ox-api-entry__description">Resolved SSG options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">11 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved SSG options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedSsgOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L276-L288" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedssgoptions-bare">
  <td><code>bare</code></td>
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
<tr id="resolvedssgoptions-lastupdated">
  <td><code>lastUpdated</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-navigation">
  <td><code>navigation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgnavigationgroup">SsgNavigationGroup</a>[]</code></td>
  <td></td>
</tr>
<tr id="resolvedssgoptions-ogimage">
  <td><code>ogImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L754-L757" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1800-L1806" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L854-L881" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2030-L2036" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1981-L1999" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1917-L1965" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L2004-L2025" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgOptions</code><span class="ox-api-entry__description">Static Site Generation options. These options control the HTML files emitted at…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">11 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Static Site Generation options.</p>
<p>These options control the HTML files emitted at build time and the matching dev-server preview behavior. Pass <code>false</code> to the top-level <code>ssg</code> option to disable the whole SSG pipeline, or pass an object to customize the defaults.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L156-L271" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgoptions-bare">
  <td><code>bare</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Emit bare HTML with only the rendered Markdown body.<br><br>This skips the default navigation, layout shell, and theme styles. It is<br>mainly useful for benchmarking, fixture generation, or projects that wrap<br>the output in their own shell.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
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
  <td><div class="ox-api-entry__member-description">Generate one Open Graph image per page.<br><br>Generated images are written alongside the SSG output and referenced from<br>each page&#39;s metadata. Configure rendering details with the top-level<br><code>ogImageOptions</code> option.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-lastupdated">
  <td><code>lastUpdated</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Add each page&#39;s last git commit timestamp to the default theme.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="ssgoptions-navigation">
  <td><code>navigation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#ssgnavigationgroup">SsgNavigationGroup</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Sidebar navigation override.<br><br>When omitted, ox-content derives navigation from the Markdown file tree.<br>Provide this when migrating from systems such as VitePress where navigation<br>is intentionally hand-authored.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
</tr>
<tr id="ssgoptions-ogimage">
  <td><code>ogImage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Static Open Graph image URL used for social sharing.<br><br>When <code>generateOgImage</code> is enabled, this value is still useful as a fallback<br>for pages that cannot produce a generated image.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">undefined</code></div></td>
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
<tr id="ssgoptions-theme">
  <td><code>theme</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./theme.md#themeconfig">ThemeConfig</a></code></td>
  <td><div class="ox-api-entry__member-description">Theme configuration for generated pages.<br><br>Use <code>defineTheme()</code> to build this object so custom theme modules and the<br>default theme extension points keep their expected shape.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">defaultTheme</code></div></td>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1811-L1817" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1297-L1317" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1237-L1252" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L1267-L1292" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/types.ts#L740-L749" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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

