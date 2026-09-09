use ox_content_mermaid::transform_mermaid;
use std::fmt::Write as _;
use std::io::Write as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).ok_or("Usage: render /path/to/mmdc [count] [distinct]")?;
    let count = args.get(2).and_then(|v| v.parse::<usize>().ok()).unwrap_or(1);
    let distinct = args.get(3).is_some_and(|v| v == "distinct");
    let mut input = String::new();
    for i in 0..count {
        write!(
            input,
            "<pre><code class=\"language-mermaid\">graph TD; A--&gt;B{};</code></pre>",
            if distinct { i } else { 0 }
        )?;
    }
    let start = std::time::Instant::now();
    let result = transform_mermaid(input, command);
    writeln!(
        std::io::stdout().lock(),
        "{{\"count\":{count},\"distinct\":{distinct},\"ms\":{},\"errors\":{},\"svg_count\":{}}}",
        start.elapsed().as_secs_f64() * 1000.0,
        result.errors.len(),
        result.html.matches("<svg").count()
    )?;
    if let Ok(output) = std::env::var("OX_MERMAID_OUTPUT") {
        std::fs::write(output, &result.html)?;
    }
    if !result.errors.is_empty() {
        return Err(result.errors.join("\n").into());
    }
    Ok(())
}
