use napi_derive::napi;

#[napi(object)]
pub struct MermaidTransformResult {
    /// The transformed HTML with mermaid code blocks replaced by rendered SVGs.
    pub html: String,
    /// Non-fatal errors encountered during rendering (per-diagram).
    pub errors: Vec<String>,
}

/// Transforms mermaid code blocks in HTML to rendered SVG diagrams.
///
/// Extracts `<pre><code class="language-mermaid">...</code></pre>` blocks,
/// renders each in parallel using the mmdc CLI, and replaces them with
/// `<div class="ox-mermaid">...</div>`.
#[napi]
pub fn transform_mermaid(html: String, mmdc_path: String) -> MermaidTransformResult {
    let blocks = extract_mermaid_blocks_from_html(&html);

    if blocks.is_empty() {
        return MermaidTransformResult { html, errors: vec![] };
    }

    // Render all diagrams in parallel using scoped threads.
    // The intermediate collect() is intentional: we must spawn ALL threads before
    // joining any, otherwise they would run sequentially instead of in parallel.
    #[allow(clippy::needless_collect)]
    let render_results: Vec<std::result::Result<String, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = blocks
            .iter()
            .map(|block| {
                let source = &block.source;
                let path = &mmdc_path;
                s.spawn(move || render_mermaid_with_mmdc(source, path))
            })
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("Thread panicked".to_string())))
            .collect()
    });

    // Replace blocks in reverse order to preserve positions
    let mut result_html = html;
    let mut errors = Vec::new();

    for (i, block) in blocks.iter().enumerate().rev() {
        match &render_results[i] {
            Ok(svg) => {
                let replacement = format!(r#"<div class="ox-mermaid">{svg}</div>"#);
                result_html.replace_range(block.start..block.end, &replacement);
            }
            Err(e) => {
                errors.push(e.clone());
            }
        }
    }

    MermaidTransformResult { html: result_html, errors }
}

struct MermaidBlock {
    start: usize,
    end: usize,
    source: String,
}

fn extract_mermaid_blocks_from_html(html: &str) -> Vec<MermaidBlock> {
    let open = r#"<pre><code class="language-mermaid">"#;
    let close = "</code></pre>";
    let mut blocks = Vec::new();
    let mut cursor = 0;

    while let Some(rel) = html[cursor..].find(open) {
        let abs_start = cursor + rel;
        let content_start = abs_start + open.len();

        if let Some(rel_end) = html[content_start..].find(close) {
            let abs_end = content_start + rel_end + close.len();
            let raw = &html[content_start..content_start + rel_end];
            blocks.push(MermaidBlock {
                start: abs_start,
                end: abs_end,
                source: decode_html_entities_mermaid(raw),
            });
            cursor = abs_end;
        } else {
            break;
        }
    }

    blocks
}

fn decode_html_entities_mermaid(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        // Numeric character references (hex)
        .replace("&#x3C;", "<")
        .replace("&#x3c;", "<")
        .replace("&#x3E;", ">")
        .replace("&#x3e;", ">")
        .replace("&#x22;", "\"")
        .replace("&#x27;", "'")
        // Numeric character references (decimal)
        .replace("&#60;", "<")
        .replace("&#62;", ">")
        .replace("&#34;", "\"")
}

static MERMAID_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn render_mermaid_with_mmdc(source: &str, mmdc_path: &str) -> std::result::Result<String, String> {
    use std::sync::atomic::Ordering;

    let temp_dir = std::env::temp_dir();
    let id = MERMAID_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();

    let input_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}.mmd"));
    let output_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}.svg"));
    let puppeteer_config_path = temp_dir.join(format!("ox_mermaid_{pid}_{id}_puppeteer.json"));

    // Write mermaid source to temp file
    std::fs::write(&input_path, source).map_err(|e| format!("Failed to write temp file: {e}"))?;

    // Write puppeteer config with --no-sandbox for CI environments
    std::fs::write(
        &puppeteer_config_path,
        r#"{"args":["--no-sandbox","--disable-setuid-sandbox"]}"#,
    )
    .map_err(|e| format!("Failed to write puppeteer config: {e}"))?;

    // Call mmdc CLI
    let output = std::process::Command::new(mmdc_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-t")
        .arg("neutral")
        .arg("-q")
        .arg("-p")
        .arg(&puppeteer_config_path)
        .output()
        .map_err(|e| {
            format!("Failed to execute mmdc: {e}. Is @mermaid-js/mermaid-cli installed?")
        })?;

    // Clean up input and puppeteer config
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&puppeteer_config_path);

    if !output.status.success() {
        let _ = std::fs::remove_file(&output_path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mmdc failed: {stderr}"));
    }

    // Read rendered SVG
    let svg = std::fs::read_to_string(&output_path)
        .map_err(|e| format!("Failed to read SVG output: {e}"))?;

    let _ = std::fs::remove_file(&output_path);

    // Post-process SVG
    let svg = postprocess_mermaid_svg(&svg, id);

    Ok(svg)
}

/// Post-process mermaid SVG output:
/// - Replace `background-color: white` with `transparent` for dark mode compatibility
/// - Replace all `my-svg` references with unique IDs to avoid collisions between diagrams
///   (covers the SVG id, CSS selectors, and marker id prefixes like `my-svg_flowchart-v2-pointEnd`)
fn postprocess_mermaid_svg(svg: &str, id: u64) -> String {
    let unique_id = format!("ox-mermaid-{id}");

    svg.replace("background-color: white;", "background-color: transparent;")
        .replace("background-color:white;", "background-color:transparent;")
        .replace("my-svg", &unique_id)
}

// ── i18n ──────────────────────────────────────────────────────
