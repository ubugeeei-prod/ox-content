use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn block(source: &str) -> String {
    format!(r#"<pre><code class="language-mermaid">{source}</code></pre>"#)
}

#[test]
fn repeated_sources_render_once_with_distinct_svg_ids() {
    let calls = AtomicUsize::new(0);
    let input = format!("İ前{}後", block("graph TD; A--&gt;B;").repeat(64));
    let output = transform_with_renderer(input, &|source| {
        calls.fetch_add(1, Ordering::Relaxed);
        assert_eq!(source, "graph TD; A-->B;");
        Ok(r#"<svg id="my-svg"><style>#my-svg{background-color: white;}</style><path marker-end="url(#my-svg-end)"/></svg>"#.to_string())
    });
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(output.errors.is_empty());
    assert!(output.html.starts_with("İ前"));
    assert!(output.html.ends_with('後'));
    assert_eq!(output.html.matches("<svg").count(), 64);
    assert!(!output.html.contains("my-svg"));
    let mut ids = Vec::new();
    for part in output.html.split("<svg id=\"").skip(1) {
        let id = part.split('"').next().unwrap_or_default();
        assert!(part.contains(&format!("#{id}{{background-color: transparent;}}")));
        assert!(part.contains(&format!("url(#{id}-end)")));
        ids.push(id);
    }
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 64);
}

#[test]
fn decoded_equivalent_sources_share_rendering_but_distinct_sources_do_not() {
    let calls = AtomicUsize::new(0);
    let output = transform_with_renderer(
        block("A--&gt;B") + &block("A--&#62;B") + &block("B--&gt;C"),
        &|_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok("<svg id=\"my-svg\"></svg>".to_string())
        },
    );
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert_eq!(output.html.matches("<svg").count(), 3);
}

#[test]
fn repeated_failures_keep_each_diagnostic_and_authored_block_in_original_error_order() {
    let calls = AtomicUsize::new(0);
    let input = block("bad A") + &block("bad A") + &block("bad B");
    let output = transform_with_renderer(input.clone(), &|source| {
        calls.fetch_add(1, Ordering::Relaxed);
        Err(source.to_string())
    });
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert_eq!(output.html, input);
    assert_eq!(output.errors, ["bad B", "bad A", "bad A"]);
}

#[test]
fn separate_documents_retry_and_absent_or_unclosed_blocks_never_render() {
    let calls = AtomicUsize::new(0);
    let renderer = |_: &str| {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok("<svg/>".to_string())
    };
    for _ in 0..2 {
        transform_with_renderer(block("A"), &renderer);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    for input in ["<p>plain</p>", r#"<pre><code class="language-mermaid">unclosed"#] {
        assert_eq!(transform_with_renderer(input.to_string(), &renderer).html, input);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}
