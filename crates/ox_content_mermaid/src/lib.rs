/// Mermaid transform result.
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
pub fn transform_mermaid(html: String, mmdc_path: &str) -> MermaidTransformResult {
    transform_with_renderer(html, &|source| render_mermaid_with_mmdc(source, mmdc_path))
}

fn transform_with_renderer(
    html: String,
    renderer: &(impl Fn(&str) -> Result<String, String> + Sync),
) -> MermaidTransformResult {
    let blocks = extract_mermaid_blocks_from_html(&html);
    if blocks.is_empty() {
        return MermaidTransformResult { html, errors: vec![] };
    }

    // Deduplicate decoded sources only within this document. Render raw SVGs;
    // each occurrence receives fresh IDs when inserted into the document.
    let mut sources: Vec<&str> = blocks.iter().map(|block| block.source.as_str()).collect();
    sources.sort_unstable();
    sources.dedup();
    #[allow(clippy::needless_collect)]
    let rendered: Vec<Result<String, String>> = std::thread::scope(|scope| {
        let handles: Vec<_> =
            sources.iter().map(|source| scope.spawn(move || renderer(source))).collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or_else(|_| Err("Thread panicked".to_string())))
            .collect()
    });

    let mut output = String::with_capacity(html.len());
    let mut cursor = 0;
    let mut errors = Vec::new();
    for block in &blocks {
        output.push_str(&html[cursor..block.start]);
        let result = sources
            .binary_search(&block.source.as_str())
            .ok()
            .and_then(|index| rendered.get(index));
        match result {
            Some(Ok(svg)) => {
                let id = MERMAID_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                output.push_str(r#"<div class="ox-mermaid">"#);
                output.push_str(&postprocess_mermaid_svg(svg, id));
                output.push_str("</div>");
            }
            Some(Err(error)) => {
                errors.push(error.clone());
                output.push_str(&html[block.start..block.end]);
            }
            None => output.push_str(&html[block.start..block.end]),
        }
        cursor = block.end;
    }
    output.push_str(&html[cursor..]);
    // Preserve the previous reverse replacement order for diagnostics.
    errors.reverse();
    MermaidTransformResult { html: output, errors }
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

#[cfg(test)]
mod tests;
