use memchr::memmem;

use super::{LastUpdatedView, SsgConfig, TocEntry};

pub(super) fn wrap_css_section(name: &str, css: &str) -> String {
    if css.trim().is_empty() {
        return String::new();
    }

    // These markers let `externalize_shared_page_assets` split one generated
    // `<style>` block into stable shared chunks. They are CSS comments, so the
    // page remains valid even before the extraction pass runs.
    format!("/* ox-content:css:{name}:start */\n{css}\n/* ox-content:css:{name}:end */\n")
}

pub(super) fn page_content_contains_any(content: &str, needles: &[&str]) -> bool {
    // Page asset detection checks for a small set of HTML markers in the final
    // content. `memmem` keeps each probe on optimized byte search instead of
    // Rust's substring machinery, and the caller short-circuits as soon as one
    // marker is present.
    let haystack = content.as_bytes();
    needles.iter().any(|needle| memmem::find(haystack, needle.as_bytes()).is_some())
}

pub(super) fn escape_html(value: &str) -> String {
    // SSG escaping favors predictable allocation over chained `replace()`:
    // allocate once at the input length, stream characters into the output,
    // and let `String` grow only if replacements make the result longer.
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
    output
}

pub(super) fn generate_toc_html(toc: &[TocEntry]) -> String {
    let mut html = String::new();

    for entry in toc {
        let depth = entry.depth.clamp(1, 6);
        html.push_str("        <li class=\"toc-item\"><a href=\"#");
        html.push_str(&escape_html(&entry.slug));
        html.push_str("\" class=\"toc-link toc-link--depth-");
        html.push_str(&depth.to_string());
        html.push_str("\">");
        html.push_str(&escape_html(&entry.text));
        html.push_str("</a></li>\n");
    }

    html
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year, month, day)
}

pub(super) fn format_last_updated(timestamp_ms: i64) -> Option<LastUpdatedView> {
    if timestamp_ms < 0 {
        return None;
    }

    let days = (timestamp_ms / 1_000).div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let date = format!("{year:04}-{month:02}-{day:02}");
    Some(LastUpdatedView { text: date.clone(), datetime: date })
}

fn infer_locale_dir(locale: &str) -> &'static str {
    match locale.split('-').next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "ar" | "fa" | "he" | "iw" | "ps" | "sd" | "ug" | "ur" | "yi" => "rtl",
        _ => "ltr",
    }
}

pub(super) fn html_locale_attrs(config: &SsgConfig) -> (&str, &str) {
    let lang = config
        .locale
        .as_deref()
        .filter(|locale| !locale.trim().is_empty())
        .or_else(|| config.available_locales.as_ref()?.first().map(|locale| locale.code.as_str()))
        .unwrap_or("en");
    let configured_dir = config.available_locales.as_ref().and_then(|locales| {
        locales.iter().find(|locale| locale.code == lang).and_then(|locale| {
            match locale.dir.as_str() {
                "ltr" => Some("ltr"),
                "rtl" => Some("rtl"),
                _ => None,
            }
        })
    });

    (lang, configured_dir.unwrap_or_else(|| infer_locale_dir(lang)))
}
