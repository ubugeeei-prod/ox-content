use super::merge_highlighted_code_blocks;

#[test]
fn merges_annotation_metadata_into_highlighted_html() {
    let original = r#"<p>Before</p><pre class="ox-code-block ox-code-block--annotated"><code class="language-ts"><span class="line ox-code-line ox-code-line--highlight" data-line="1">const first = 1;</span>
<span class="line ox-code-line ox-code-line--warning" data-line="2">const second = 2;</span>
<span class="line ox-code-line ox-code-line--error" data-line="3">throw new Error("boom");</span>
</code></pre><p>After</p>"#;
    let highlighted = r#"<p>Before</p><pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">const first = 1;</span></span>
<span class="line"><span style="color:#E1E4E8">const second = 2;</span></span>
<span class="line"><span style="color:#E1E4E8">throw new Error("boom");</span></span>
</code></pre><p>After</p>"#;

    let merged = merge_highlighted_code_blocks(original, highlighted);

    assert!(
        merged.contains(r#"<pre class="shiki github-dark ox-code-block ox-code-block--annotated""#)
    );
    assert!(merged.contains(r#"class="language-ts" data-language="ts""#));
    assert!(merged.contains(r#"class="line ox-code-line ox-code-line--highlight" data-line="1""#));
    assert!(merged.contains(r#"class="line ox-code-line ox-code-line--warning" data-line="2""#));
    assert!(merged.contains(r#"class="line ox-code-line ox-code-line--error" data-line="3""#));
}

#[test]
fn preserves_language_metadata_for_non_annotated_code_blocks() {
    let original = r#"<pre><code class="language-rs">fn main() {}
</code></pre>"#;
    let highlighted = r#"<pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">fn main() {}</span></span>
</code></pre>"#;

    let merged = merge_highlighted_code_blocks(original, highlighted);

    assert!(merged.contains(r#"class="language-rs" data-language="rs""#));
    assert!(merged.contains(r#"<pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0" data-language="rs">"#));
}
