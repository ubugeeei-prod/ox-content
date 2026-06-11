use super::chunk::{AssetCache, AssetKind};

const CSS_SECTION_PREFIX: &str = "/* ox-content:css:";
const CSS_SECTION_START_SUFFIX: &str = ":start */";
const CSS_SECTION_END_SUFFIX: &str = ":end */";
const CORE_CSS_SECTION_NAMES: [&str; 2] = ["base", "footer"];
const THEME_INLINE_CSS_MAX_BYTES: usize = 2048;

#[derive(Debug, Clone)]
struct CssSection {
    name: String,
    content: String,
}

pub(super) fn build_style_replacement(
    css_content: &str,
    css_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // CSS is split by `ox-content:css:*` section markers so globally shared
    // base/footer/plugin styles can become cacheable files while page-specific
    // theme overrides may stay inline. Without this split every page's full
    // `<style>` block would differ as soon as theme CSS differed.
    let sections = extract_css_sections(css_content);
    let effective_sections = if sections.is_empty() {
        vec![CssSection { name: "css".to_string(), content: css_content.trim().to_string() }]
    } else {
        sections
    };

    let core_content = effective_sections
        .iter()
        .filter(|section| CORE_CSS_SECTION_NAMES.contains(&section.name.as_str()))
        .map(|section| section.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    let mut fragments = Vec::new();
    if !core_content.is_empty() {
        let core_chunk =
            css_chunks.get_or_create(AssetKind::Css, "core", &core_content, out_dir, base);
        fragments.push(format!("  <link rel=\"stylesheet\" href=\"{}\">", core_chunk.public_path));
    }

    for section in effective_sections {
        if CORE_CSS_SECTION_NAMES.contains(&section.name.as_str()) {
            continue;
        }

        let has_relative_urls = has_relative_css_urls(&section.content);
        let should_inline_theme = section.name == "theme"
            && (has_relative_urls || section.content.len() <= THEME_INLINE_CSS_MAX_BYTES);
        if should_inline_theme || has_relative_urls {
            fragments.push(format!("  <style>{}</style>", section.content));
            continue;
        }

        let chunk = css_chunks.get_or_create(
            AssetKind::Css,
            &section.name,
            &section.content,
            out_dir,
            base,
        );
        fragments.push(format!("  <link rel=\"stylesheet\" href=\"{}\">", chunk.public_path));
    }

    fragments.join("\n")
}

fn extract_css_sections(css_content: &str) -> Vec<CssSection> {
    // Section markers are emitted as CSS comments, so they survive template
    // rendering and minification-free builds. Parse with string searches rather
    // than regex to keep this pass cheap over the full page stylesheet.
    let mut sections = Vec::new();
    let mut cursor = 0;

    while let Some(start_rel) = css_content[cursor..].find(CSS_SECTION_PREFIX) {
        let marker_start = cursor + start_rel;
        let name_start = marker_start + CSS_SECTION_PREFIX.len();
        let Some(name_end_rel) = css_content[name_start..].find(CSS_SECTION_START_SUFFIX) else {
            break;
        };
        let name_end = name_start + name_end_rel;
        let name = &css_content[name_start..name_end];
        if !name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
            cursor = name_end;
            continue;
        }

        let content_start = name_end + CSS_SECTION_START_SUFFIX.len();
        let end_marker = format!("{CSS_SECTION_PREFIX}{name}{CSS_SECTION_END_SUFFIX}");
        let Some(end_rel) = css_content[content_start..].find(&end_marker) else {
            break;
        };
        let content_end = content_start + end_rel;
        let content = css_content[content_start..content_end].trim();
        if !content.is_empty() {
            sections.push(CssSection { name: name.to_string(), content: content.to_string() });
        }
        cursor = content_end + end_marker.len();
    }

    sections
}

fn has_relative_css_urls(css: &str) -> bool {
    // Relative URLs are resolved relative to the HTML page when CSS is inline,
    // but relative to `/assets/...` after extraction. Detect those sections and
    // leave them inline rather than rewriting every CSS URL.
    let mut cursor = 0;
    let bytes = css.as_bytes();

    while let Some(url_rel) = css[cursor..].find("url(") {
        let url_index = cursor + url_rel;
        let mut value_start = url_index + 4;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }

        let quote = match bytes.get(value_start).copied() {
            Some(b'"' | b'\'') => {
                let quote = bytes[value_start];
                value_start += 1;
                Some(quote)
            }
            _ => None,
        };

        let mut value_end = value_start;
        while value_end < bytes.len() {
            let byte = bytes[value_end];
            if let Some(quote) = quote {
                if byte == b'\\' {
                    value_end = value_end.saturating_add(2);
                    continue;
                }
                if byte == quote {
                    break;
                }
            } else if byte == b')' {
                break;
            }
            value_end += 1;
        }

        let value = css[value_start..value_end].trim();
        if !value.is_empty()
            && !value.starts_with("data:")
            && !value.starts_with("http:")
            && !value.starts_with("https:")
            && !value.starts_with("//")
            && !value.starts_with('/')
            && !value.starts_with('#')
            && !value.starts_with("blob:")
            && !value.starts_with("var(")
        {
            return true;
        }

        cursor = value_end.saturating_add(1);
    }

    false
}
