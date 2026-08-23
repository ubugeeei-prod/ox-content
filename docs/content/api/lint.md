# lint.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts)**

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
  <strong>6</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>6</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>31</strong>
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

<details id="lintmarkdown" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">lintMarkdown(source: string, options: MarkdownLintOptions = {}): MarkdownLintResult</code><span class="ox-api-entry__description">Lints Markdown prose with the Rust-backed built-in rule engine.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns MarkdownLintResult</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Lints Markdown prose with the Rust-backed built-in rule engine.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function lintMarkdown(source: string, options: MarkdownLintOptions = {}): MarkdownLintResult</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L299-L305" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">source</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#markdownlintoptions">MarkdownLintOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#markdownlintresult">MarkdownLintResult</a></code>
  
</div>
</div>
  </div>
</details>

<details id="lintmarkdownasync" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">lintMarkdownAsync(source: string, options: MarkdownLintOptions = {}): Promise&lt;MarkdownLintResult&gt;</code><span class="ox-api-entry__description">Async Markdown linter that supports opt-in standard dictionaries.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;MarkdownLintResult&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Async Markdown linter that supports opt-in standard dictionaries.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function lintMarkdownAsync(source: string, options: MarkdownLintOptions = {}): Promise&lt;MarkdownLintResult&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L310-L317" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">source</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#markdownlintoptions">MarkdownLintOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#markdownlintresult">MarkdownLintResult</a>&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="lintmarkdowndocumentsasync" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">lintMarkdownDocumentsAsync(sources: string[], options: MarkdownLintOptions = {}): Promise&lt;MarkdownLintResult[]&gt;</code><span class="ox-api-entry__description">Internal batched Markdown linting entry point used by file-based workflows.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;MarkdownLintResult[]&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Internal batched Markdown linting entry point used by file-based workflows.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function lintMarkdownDocumentsAsync(sources: string[], options: MarkdownLintOptions = {}): Promise&lt;MarkdownLintResult[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L322-L328" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">sources</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#markdownlintoptions">MarkdownLintOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: {}</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#markdownlintresult">MarkdownLintResult</a>[]&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="markdownlintdiagnostic" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintDiagnostic</code><span class="ox-api-entry__description">A single Markdown lint diagnostic.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A single Markdown lint diagnostic.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintDiagnostic</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L179-L224" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintdiagnostic-column">
  <td><code>column</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-indexed start column.</div></td>
</tr>
<tr id="markdownlintdiagnostic-endcolumn">
  <td><code>endColumn</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-indexed end column.</div></td>
</tr>
<tr id="markdownlintdiagnostic-endline">
  <td><code>endLine</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-indexed end line.</div></td>
</tr>
<tr id="markdownlintdiagnostic-language">
  <td><code>language</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownLintLanguage</code></td>
  <td><div class="ox-api-entry__member-description">Language used for spellchecking, when relevant.</div></td>
</tr>
<tr id="markdownlintdiagnostic-line">
  <td><code>line</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">1-indexed line number.</div></td>
</tr>
<tr id="markdownlintdiagnostic-message">
  <td><code>message</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Human-readable explanation.</div></td>
</tr>
<tr id="markdownlintdiagnostic-ruleid">
  <td><code>ruleId</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Stable rule identifier.</div></td>
</tr>
<tr id="markdownlintdiagnostic-severity">
  <td><code>severity</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownLintSeverity</code></td>
  <td><div class="ox-api-entry__member-description">Diagnostic severity.</div></td>
</tr>
<tr id="markdownlintdiagnostic-suggestions">
  <td><code>suggestions</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Suggested replacements, when available.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownlintdictionaryoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintDictionaryOptions</code><span class="ox-api-entry__description">Additional dictionary configuration for the Markdown linter.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Additional dictionary configuration for the Markdown linter.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintDictionaryOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L72-L99" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintdictionaryoptions-bylanguage">
  <td><code>byLanguage</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Partial&lt;Record&lt;MarkdownLintLanguage, string[]&gt;&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Extra words to allow per language.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{}</code></div></td>
</tr>
<tr id="markdownlintdictionaryoptions-ignoredwords">
  <td><code>ignoredWords</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Words that should never produce diagnostics.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="markdownlintdictionaryoptions-standard">
  <td><code>standard</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdownlintstandarddictionaryoptions">MarkdownLintStandardDictionaryOptions</a> | false</code></td>
  <td><div class="ox-api-entry__member-description">Opt-in standard dictionary datasets.<br><br>By default the linter stays on a minimal built-in dictionary. Enable this<br>to load larger locale dictionaries from a standard external source.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">false</code></div></td>
</tr>
<tr id="markdownlintdictionaryoptions-words">
  <td><code>words</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Words ignored across all configured languages.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownlintoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintOptions</code><span class="ox-api-entry__description">Options for linting Markdown documents.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Options for linting Markdown documents.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L151-L174" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintoptions-dictionary">
  <td><code>dictionary</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdownlintdictionaryoptions">MarkdownLintDictionaryOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Built-in and opt-in standard dictionary overrides.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{}</code></div></td>
</tr>
<tr id="markdownlintoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownLintLanguage[]</code></td>
  <td><div class="ox-api-entry__member-description">Languages enabled for spellchecking.<br><br>When <code>dictionary.standard.languages</code> is provided and this option is<br>omitted, those languages are used instead.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[&#39;en&#39;]</code></div></td>
</tr>
<tr id="markdownlintoptions-rules">
  <td><code>rules</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdownlintruleoptions">MarkdownLintRuleOptions</a></code></td>
  <td><div class="ox-api-entry__member-description">Rule configuration.<br>Omitted fields use <code>MarkdownLintRuleOptions</code> defaults.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">{}</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownlintresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintResult</code><span class="ox-api-entry__description">Markdown lint report.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Markdown lint report.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintResult</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L229-L249" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintresult-diagnostics">
  <td><code>diagnostics</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#markdownlintdiagnostic">MarkdownLintDiagnostic</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">All collected diagnostics.</div></td>
</tr>
<tr id="markdownlintresult-errorcount">
  <td><code>errorCount</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of error diagnostics.</div></td>
</tr>
<tr id="markdownlintresult-infocount">
  <td><code>infoCount</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of info diagnostics.</div></td>
</tr>
<tr id="markdownlintresult-warningcount">
  <td><code>warningCount</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of warning diagnostics.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownlintruleoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintRuleOptions</code><span class="ox-api-entry__description">Rule switches for Markdown linting.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rule switches for Markdown linting.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintRuleOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L104-L146" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintruleoptions-duplicateheadings">
  <td><code>duplicateHeadings</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report headings that repeat the same visible text.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="markdownlintruleoptions-headingincrement">
  <td><code>headingIncrement</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report heading depth jumps such as <code>#</code> -&gt; <code>###</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="markdownlintruleoptions-maxconsecutiveblanklines">
  <td><code>maxConsecutiveBlankLines</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Maximum number of blank lines allowed in a row.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">1</code></div></td>
</tr>
<tr id="markdownlintruleoptions-repeatedpunctuation">
  <td><code>repeatedPunctuation</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report duplicated terminal punctuation such as <code>!!</code> or <code>？？</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="markdownlintruleoptions-repeatedwords">
  <td><code>repeatedWords</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report adjacent repeated words in visible prose.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="markdownlintruleoptions-spellcheck">
  <td><code>spellcheck</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable built-in multilingual spellchecking.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="markdownlintruleoptions-trailingspaces">
  <td><code>trailingSpaces</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Report trailing spaces.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="markdownlintstandarddictionaryoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MarkdownLintStandardDictionaryOptions</code><span class="ox-api-entry__description">Opt-in standard dictionary sources. The default provider uses CSpell dictionary…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in standard dictionary sources.</p>
<p>The default provider uses CSpell dictionary packages because those packages are actively maintained and expose locale-specific dictionaries in a stable config format. Languages without a bundled preset can still be added through custom <code>imports</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface MarkdownLintStandardDictionaryOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/lint.ts#L35-L67" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="markdownlintstandarddictionaryoptions-imports">
  <td><code>imports</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Additional CSpell-compatible imports.<br><br>This can point at installed packages like<br><code>@cspell/dict-fr-fr/cspell-ext.json</code> or local CSpell config files.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="markdownlintstandarddictionaryoptions-languages">
  <td><code>languages</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">MarkdownLintLanguage[]</code></td>
  <td><div class="ox-api-entry__member-description">Languages whose default standard dictionaries should be enabled.<br><br>Built-in preset package mappings currently exist for <code>en</code>, <code>fr</code>, <code>de</code>,<br>and <code>pl</code>. For other languages, use <code>imports</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">[]</code></div></td>
</tr>
<tr id="markdownlintstandarddictionaryoptions-provider">
  <td><code>provider</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;cspell&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Standard dictionary provider.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&quot;cspell&quot;</code></div></td>
</tr>
<tr id="markdownlintstandarddictionaryoptions-resolveimportsrelativeto">
  <td><code>resolveImportsRelativeTo</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string | URL</code></td>
  <td><div class="ox-api-entry__member-description">Base URL or path used when resolving <code>imports</code>.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">new URL(&quot;.&quot;, import.meta.url)</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

