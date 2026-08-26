use super::merge_highlighted_code_blocks;

#[test]
fn merges_annotation_metadata_into_highlighted_html() {
    let original = r#"<p>Before</p><pre class="ox-code-block ox-code-block--annotated"><code class="language-ts"><span class="line ox-code-line ox-code-line--highlight" data-line="1">const first = 1;</span>
<span class="line ox-code-line ox-code-line--warning" data-line="2">const second = 2;</span>
<span class="line ox-code-line ox-code-line--error" data-line="3">throw new Error("boom");</span>
</code></pre><p>After</p>"#;
    let highlighted = r#"<p>Before</p><pre class="ox-highlight github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">const first = 1;</span></span>
<span class="line"><span style="color:#E1E4E8">const second = 2;</span></span>
<span class="line"><span style="color:#E1E4E8">throw new Error("boom");</span></span>
</code></pre><p>After</p>"#;

    let merged = merge_highlighted_code_blocks(original, highlighted);

    insta::assert_snapshot!(merged);
}

#[test]
fn preserves_language_metadata_for_non_annotated_code_blocks() {
    let original = r#"<pre><code class="language-rs">fn main() {}
</code></pre>"#;
    let highlighted = r#"<pre class="ox-highlight github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">fn main() {}</span></span>
</code></pre>"#;

    let merged = merge_highlighted_code_blocks(original, highlighted);

    insta::assert_snapshot!(merged);
}

#[test]
fn preserves_line_link_targets_after_highlighting() {
    let original = r#"<pre class="ox-code-block ox-code-block--line-links" data-line-link-prefix="auth-loader"><code class="language-ts"><span class="line ox-code-line" id="auth-loader-L27" data-line="1" data-line-anchor="auth-loader-L27">const token = readToken();</span>
<span class="line ox-code-line" id="auth-loader-L28" data-line="2" data-line-anchor="auth-loader-L28">return token;</span></code></pre>"#;
    let highlighted = r#"<pre class="ox-highlight github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">const token = readToken();</span></span>
<span class="line"><span style="color:#E1E4E8">return token;</span></span></code></pre>"#;

    let merged = merge_highlighted_code_blocks(original, highlighted);

    assert!(merged.contains(r#"data-line-link-prefix="auth-loader""#), "{merged}");
    assert!(merged.contains(r#"id="auth-loader-L27""#), "{merged}");
    assert!(merged.contains(r#"data-line-anchor="auth-loader-L28""#), "{merged}");
}

#[test]
fn decodes_entities_next_to_multibyte_text() {
    // Regression: the entity scan capped its search by byte length and then
    // sliced the string, which split a multi-byte character and panicked.
    let block = "<pre><code class=\"language-ts\">日本語 &lt; テキスト &amp; more</code></pre>";
    let result = super::highlight_code_blocks(
        block,
        |_| true,
        |code, lang| {
            assert_eq!(lang, "ts");
            assert_eq!(code, "日本語 < テキスト & more");
            Some(format!("<pre><code>{code}</code></pre>"))
        },
    );

    assert!(result.skipped.is_empty());
    assert!(result.html.contains("日本語 < テキスト & more"));
}

#[test]
fn recovers_the_source_of_a_block_wrapped_in_annotation_spans() {
    // Annotated blocks reach the highlighter already carrying `<span>` line
    // wrappers. Declining them would send the page back through the HTML
    // round trip, so the wrappers are stripped to recover the source instead.
    let block =
        "<pre><code class=\"language-ts\"><span class=\"line\">const a = 1;</span></code></pre>";
    let result = super::highlight_code_blocks(
        block,
        |_| true,
        |code, _| {
            assert_eq!(code, "const a = 1;");
            Some("<pre><code>highlighted</code></pre>".to_string())
        },
    );

    assert!(result.skipped.is_empty());
    assert!(result.html.contains("highlighted"));
}

#[test]
fn recovers_the_source_of_a_member_type_that_cross_references_another() {
    // The docs generator links a type name to its definition from inside the
    // member type. The rehype pass read straight through the link, so the
    // highlighted text must come out the same way.
    let html =
        "<p><code class=\"mt language-ts\"><a href=\"./x.html\">NavGroup</a>[] | null</code></p>";
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |code, _| {
            assert_eq!(code, "NavGroup[] | null");
            Some("<pre><code><span>t</span></code></pre>".to_string())
        },
    );

    assert!(result.skipped.is_empty());
    assert!(result.html.contains("ox-highlight-inline"));
}

#[test]
fn declines_a_block_whose_code_holds_more_than_text_wrappers() {
    // A JSDoc `@example` that was itself run through the Markdown renderer
    // arrives with block elements nested inside `<code>`. That is not
    // well-formed, and a text scan and a real HTML parser disagree about what
    // it means, so the DOM path stays the authority on it.
    let block = "<pre><code class=\"language-ts\">a\n<p>const b = 1;</p></code></pre>";
    let result =
        super::highlight_code_blocks(block, |_| true, |_, _| Some("<pre>x</pre>".to_string()));

    assert_eq!(result.skipped, ["ts"]);
    assert_eq!(result.html, block);
}

#[test]
fn hands_an_unsupported_language_to_the_caller_instead_of_surrendering() {
    // The block is well formed, so only it needs another highlighter — the
    // rest of the page is still done here.
    let html = "<pre><code class=\"language-vue\">x</code></pre>\n\
                <pre><code class=\"language-ts\">y</code></pre>";
    let result = super::highlight_code_blocks(
        html,
        |lang| lang != "vue",
        |_, _| Some("<pre><code>done</code></pre>".to_string()),
    );

    assert!(result.skipped.is_empty(), "a well-formed block must not surrender the page");
    assert_eq!(result.pending.len(), 1);
    assert_eq!(result.pending[0].language, "vue");
    assert_eq!(result.pending[0].source, "x");
    assert!(result.html.contains("done"), "the supported block is still highlighted");
    assert!(result.html.contains("language-vue"), "the pending block is left in place");
}

#[test]
fn splices_the_callers_highlighting_back_over_a_pending_block() {
    let html = "<pre><code class=\"language-vue\">x</code></pre>";
    let merged = super::apply_pending_highlights(
        html,
        &["<pre class=\"ox-highlight\"><code><span>x</span></code></pre>".to_string()],
        |lang| lang != "vue",
    );

    assert!(merged.contains("ox-highlight"));
    assert!(merged.contains("<span>x</span>"));
    assert!(merged.contains("data-language=\"vue\""));
}

#[test]
fn leaves_the_blocks_it_already_highlighted_out_of_the_second_pass() {
    // The two scans are lined up by `supports` alone: the first pass highlights
    // what it claims and lists the rest, and the second must walk to the same
    // elements in the same order. Touching a claimed one would consume a
    // replacement meant for a later block and shift every one after it.
    let html = "<pre class=\"ox-highlight\"><code class=\"language-ts\"><span>done</span></code></pre>\n\
                <pre><code class=\"language-vue\">x</code></pre>";
    let merged = super::apply_pending_highlights(
        html,
        &["<pre class=\"ox-highlight\"><code><span>vue</span></code></pre>".to_string()],
        |lang| lang != "vue",
    );

    assert!(merged.contains("<span>done</span>"), "the claimed block must be untouched");
    assert_eq!(merged.matches("<span>done</span>").count(), 1);
    assert!(merged.contains("<span>vue</span>"), "the pending block must be replaced");
}

#[test]
fn gives_a_pending_block_its_language_even_when_nothing_could_highlight_it() {
    // An empty replacement is "no grammar for this either". The tree walk
    // reached the same state by leaving the element alone and merging the
    // original metadata back over it, which is where `data-language` came
    // from, so a block still picks that up.
    let html = "<pre><code class=\"language-mermaid\">flowchart LR</code></pre>";
    let merged = super::apply_pending_highlights(html, &[String::new()], |lang| lang != "mermaid");

    assert!(merged.contains("<pre data-language=\"mermaid\">"));
    assert!(merged.contains("flowchart LR"));
    assert!(!merged.contains("ox-highlight"));
}

#[test]
fn highlights_an_inline_element_without_giving_it_a_block_wrapper() {
    let html = "<p>see <code class=\"sig language-ts\">a: string</code></p>";
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |code, _| {
            assert_eq!(code, "a: string");
            Some(
                "<pre class=\"ox-highlight\"><code class=\"language-ts\"><span>a</span></code></pre>"
                    .to_string(),
            )
        },
    );

    assert!(result.skipped.is_empty());
    assert!(!result.html.contains("<pre"), "inline code must not gain a block wrapper");
    assert!(result.html.contains("ox-highlight-inline"));
    assert!(result.html.contains("class=\"sig language-ts ox-highlight-inline\""));
    assert!(result.html.contains("data-language=\"ts\""));
    assert!(result.html.contains("<span>a</span>"));
}

#[test]
fn leaves_inline_code_without_a_language_alone_and_does_not_report_it() {
    let html = "<p>see <code>plain</code></p>";
    let result =
        super::highlight_code_blocks(html, |_| true, |_, _| panic!("must not be highlighted"));

    assert!(result.skipped.is_empty());
    assert_eq!(result.html, html);
}

#[test]
fn highlights_a_repeated_snippet_once_and_reuses_it() {
    // API pages are mostly member types, and the same handful of type names
    // recurs on every entry.
    let html = "<p><code class=\"language-ts\">string</code> \
                <code class=\"language-ts\">string</code> \
                <code class=\"language-ts\">boolean</code></p>";
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |code, _| {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(format!("<pre><code><span>{code}</span></code></pre>"))
        },
    );

    assert!(result.skipped.is_empty());
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "the repeated snippet must be highlighted once"
    );
    assert_eq!(result.html.matches("<span>string</span>").count(), 2);
    assert_eq!(result.html.matches("<span>boolean</span>").count(), 1);
}

#[test]
fn keeps_each_element_of_a_repeated_snippet_on_its_own_classes() {
    // The highlight is shared but the merge is not: each element keeps the
    // classes it arrived with.
    let html = "<p><code class=\"first language-ts\">string</code> \
                <code class=\"second language-ts\">string</code></p>";
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |_, _| Some("<pre><code><span>t</span></code></pre>".to_string()),
    );

    assert!(result.html.contains("class=\"first language-ts ox-highlight-inline\""));
    assert!(result.html.contains("class=\"second language-ts ox-highlight-inline\""));
}

#[test]
fn highlights_nothing_at_all_once_one_element_is_out_of_reach() {
    // The caller answers a non-empty `skipped` by running its own highlighter
    // over the whole page, which redoes every block. Anything highlighted here
    // would be thrown away, so nothing is. Only markup this pass cannot read
    // does that — an unsupported language comes back as pending instead.
    let html = "<pre><code class=\"language-ts\">a</code></pre>\n                <pre><code class=\"language-ts\">b\n<p>c</p></code></pre>";
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |_, _| {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some("<pre>x</pre>".to_string())
        },
    );

    assert_eq!(result.skipped, ["ts"]);
    assert_eq!(result.html, html);
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "the claimable block must not be highlighted"
    );
}

#[test]
fn names_the_language_of_the_snippet_the_highlighter_gave_up_on() {
    // `supports` promised both languages, so a `None` here is the highlighter
    // contradicting itself. The page goes back untouched for the caller to
    // redo — and `skipped` names the language that actually failed, not
    // whichever one happened to be claimed first.
    let html = "<pre><code class=\"language-ts\">a</code></pre>\n\
                <pre><code class=\"language-rs\">b</code></pre>";
    let result = super::highlight_code_blocks(
        html,
        |_| true,
        |_, language| {
            (language != "rs").then(|| "<pre><code><span>t</span></code></pre>".to_string())
        },
    );

    assert_eq!(result.skipped, ["rs"]);
    assert_eq!(result.html, html);
    assert!(result.pending.is_empty());
}

#[test]
fn spreading_a_page_across_threads_keeps_each_result_on_its_own_element() {
    // Enough distinct source to cross `PARALLEL_SOURCE_BYTES`, so this walks
    // the threaded path. Every snippet highlights to a marker built from its
    // own text, which the elements must carry back in the order they appeared:
    // a page whose highlights landed on the wrong blocks is the failure that
    // parallelising this could plausibly introduce.
    let sources: Vec<String> =
        (0..64).map(|i| format!("let v{i:03} = {};", "a".repeat(60))).collect();
    assert!(
        sources.iter().map(String::len).sum::<usize>() >= super::blocks::PARALLEL_SOURCE_BYTES,
        "the fixture must be big enough to take the threaded path"
    );

    let mut html = String::new();
    for source in &sources {
        html.push_str("<pre><code class=\"language-ts\">");
        html.push_str(source);
        html.push_str("</code></pre>\n");
    }

    let result = super::highlight_code_blocks(
        &html,
        |_| true,
        |code, _| Some(format!("<pre><code><span>{code}</span></code></pre>")),
    );

    assert!(result.skipped.is_empty());
    let mut last = 0;
    for source in &sources {
        let marker = format!("<span>{source}</span>");
        assert_eq!(result.html.matches(&marker).count(), 1, "{source} must appear once");
        let at = result.html.find(&marker).expect("every snippet must reach the page");
        assert!(at > last, "{source} landed out of order");
        last = at;
    }
}
