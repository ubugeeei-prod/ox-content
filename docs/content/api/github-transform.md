# transform.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/transform.ts)**

> 2 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>2</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>6</strong>
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

<details id="rehypegithub" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">rehypeGitHub(repoDataMap: Map&lt;string, GitHubRepoData | null&gt;, sourceDataMap: Map&lt;string, GitHubSourceData | null&gt;, options: Required&lt;GitHubOptions&gt;)</code><span class="ox-api-entry__description">Rehype plugin to transform GitHub components.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rehype plugin to transform GitHub components.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function rehypeGitHub(repoDataMap: Map&lt;string, GitHubRepoData | null&gt;, sourceDataMap: Map&lt;string, GitHubSourceData | null&gt;, options: Required&lt;GitHubOptions&gt;)</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/transform.ts#L31-L73" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">repoDataMap</code>
    <code class="ox-api-entry__param-type">Map&lt;string, GitHubRepoData | null&gt;</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">sourceDataMap</code>
    <code class="ox-api-entry__param-type">Map&lt;string, GitHubSourceData | null&gt;</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">Required&lt;GitHubOptions&gt;</code>
  </div>
</li>
</ul>
</div>
  </div>
</details>

<details id="transformgithub" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformGitHub(html: string, repoDataMap?: Map&lt;string, GitHubRepoData | null&gt;, options?: GitHubOptions): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transform GitHub components in HTML.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transform GitHub components in HTML.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function transformGitHub(html: string, repoDataMap?: Map&lt;string, GitHubRepoData | null&gt;, options?: GitHubOptions): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/github/transform.ts#L78-L99" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
    <code class="ox-api-entry__param-name">repoDataMap</code>
    <code class="ox-api-entry__param-type">Map&lt;string, GitHubRepoData | null&gt;</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">GitHubOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string&gt;</code>
</div>
</div>
  </div>
</details>
