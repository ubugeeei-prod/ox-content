use super::{Context, inline::label_end};
use ox_content_ast::Span;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

static REFERENCES: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"!?\[([^\]]+)\](?:\[([^\]]*)\])?").ok());
static ANCHORS: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)\b(id|name)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).ok());
static LINE_FRAGMENT: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^#(?:L\d+(?:C\d+)?-L\d+(?:C\d+)?|L\d+)$").ok());

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    if ctx.settings.on(51) {
        fragments(ctx);
    }
    if ctx.settings.on(52) || ctx.settings.on(53) {
        definitions(ctx);
    }
    proper_names(ctx);
}

fn definitions(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let defined: FxHashSet<_> = doc.definitions.iter().map(|v| normalize(&v.label)).collect();
    let mut used = FxHashSet::default();
    let base = crate::lint::source::frontmatter_end(ctx.source.text);
    let Some(pattern) = REFERENCES.as_ref() else {
        return;
    };
    let mut captures = pattern.capture_locations();
    let mut offset = base;
    while let Some(whole) = pattern.captures_read_at(&mut captures, ctx.source.text, offset) {
        offset = whole.end();
        let first = captures.get(1).map_or("", |(start, end)| &ctx.source.text[start..end]);
        let span = Span::new(whole.start() as u32, whole.end() as u32);
        if ctx.source.text[whole.end()..].starts_with('(') || first.starts_with('^') {
            continue;
        }
        if whole.start() < base {
            continue;
        }
        if ctx.code_at(whole.start()).is_some()
            || super::contains(&doc.inline_code, span.start)
            || super::contains(&doc.html, span.start)
            || super::at(&doc.definitions, span.start, |v| v.span).is_some()
        {
            continue;
        }
        if whole.start() > 0 && ctx.source.text.as_bytes()[whole.start() - 1] == b'\\' {
            continue;
        }
        if ctx.source.text[whole.end()..].starts_with('(') || first.starts_with('^') {
            continue;
        }
        let explicit = captures.get(2).map(|(start, end)| &ctx.source.text[start..end]);
        let label = normalize(explicit.filter(|v| !v.is_empty()).unwrap_or(first));
        if defined.contains(&label) {
            used.insert(label);
            continue;
        }
        let ignored = ctx.settings.strings(52, "ignored_labels").map_or(label == "X", |labels| {
            labels.iter().filter_map(serde_json::Value::as_str).any(|v| normalize(v) == label)
        });
        if !ignored && (explicit.is_some() || ctx.settings.boolean(52, "shortcut_syntax", false)) {
            ctx.report(52, span);
        }
    }
    let mut seen = FxHashSet::default();
    for definition in &doc.definitions {
        let label = normalize(&definition.label);
        let duplicate = !seen.insert(label.clone());
        let ignored =
            ctx.settings.strings(53, "ignored_definitions").map_or(label == "//", |labels| {
                labels.iter().filter_map(serde_json::Value::as_str).any(|v| normalize(v) == label)
            });
        if !ignored && (duplicate || !used.contains(&label)) {
            ctx.report(53, definition.span);
        }
    }
}

fn normalize(text: &str) -> compact_str::CompactString {
    let mut value = compact_str::CompactString::default();
    for word in text.split_whitespace() {
        if !value.is_empty() {
            value.push(' ');
        }
        for c in word.chars().flat_map(char::to_uppercase) {
            value.push(c);
        }
    }
    value
}

fn fragments(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut anchors = FxHashMap::<String, usize>::default();
    anchors.insert("#top".into(), 0);
    for heading in &doc.headings {
        let mut slug = String::from("#");
        for c in heading.slug_text.chars().flat_map(char::to_lowercase) {
            if c == ' ' {
                slug.push('-');
            } else if c.is_alphanumeric()
                || c == '-'
                || c == '_'
                || unicode_normalization::char::is_combining_mark(c)
            {
                let mut buffer = [0; 4];
                for &byte in c.encode_utf8(&mut buffer).as_bytes() {
                    if byte.is_ascii() {
                        slug.push(char::from(byte));
                    } else {
                        const HEX: &[u8] = b"0123456789ABCDEF";
                        slug.push('%');
                        slug.push(char::from(HEX[(byte >> 4) as usize]));
                        slug.push(char::from(HEX[(byte & 15) as usize]));
                    }
                }
            }
        }
        if slug != "#" {
            let count = *anchors.get(&slug).unwrap_or(&0);
            if count > 0 {
                anchors.insert(format!("{slug}-{count}"), 0);
            }
            anchors.insert(slug, count + 1);
        }
        if let Some(start) = heading.text.find("{#")
            && let Some(end) = heading.text[start + 2..].find('}')
        {
            anchors.insert(format!("#{}", &heading.text[start + 2..start + 2 + end]), 0);
        }
    }
    for &html in &doc.html {
        for found in ANCHORS.as_ref().into_iter().flat_map(|re| re.captures_iter(ctx.raw(html))) {
            let raw = ctx.raw(html).trim_start();
            let name_allowed = found[1].eq_ignore_ascii_case("id")
                || raw.get(..2).is_some_and(|v| v.eq_ignore_ascii_case("<a"))
                    && raw.as_bytes().get(2).is_some_and(u8::is_ascii_whitespace);
            if name_allowed
                && let Some(label) = found.get(2).or_else(|| found.get(3)).or_else(|| found.get(4))
            {
                anchors.insert(format!("#{}", label.as_str()), 0);
            }
        }
    }
    for link in &doc.links {
        if link.image || !link.url.starts_with('#') || link.url == "#" {
            continue;
        }
        let url = link.url.as_str();
        if ctx.settings.regexes[51].as_ref().is_some_and(|re| re.is_match(url))
            || LINE_FRAGMENT.as_ref().is_some_and(|re| re.is_match(url))
        {
            continue;
        }
        let valid = if ctx.settings.boolean(51, "ignore_case", false) {
            anchors.keys().any(|anchor| anchor.to_lowercase() == url.to_lowercase())
        } else {
            anchors.contains_key(url)
        };
        if !valid {
            ctx.report(51, link.span);
        }
    }
}

fn proper_names(ctx: &mut Context<'_, '_>) {
    if !ctx.settings.on(44) {
        return;
    }
    for (name, pattern) in &ctx.settings.proper_names {
        for found in pattern.find_iter(ctx.source.text) {
            if found.as_str() == name
                || found.start() < crate::lint::source::frontmatter_end(ctx.source.text)
            {
                continue;
            }
            let before = ctx.source.text[..found.start()].chars().next_back();
            let after = ctx.source.text[found.end()..].chars().next();
            let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
            if before.is_some_and(word) || after.is_some_and(word) {
                continue;
            }
            if (ctx.code_at(found.start()).is_some()
                || ctx
                    .document
                    .inline_code
                    .iter()
                    .any(|v| v.start as usize <= found.start() && found.end() <= v.end as usize))
                && !ctx.settings.boolean(44, "code_blocks", true)
            {
                continue;
            }
            if ctx
                .document
                .html
                .iter()
                .any(|v| v.start as usize <= found.start() && found.end() <= v.end as usize)
                && !ctx.settings.boolean(44, "html_elements", true)
            {
                continue;
            }
            let destination = ctx.document.links.iter().any(|link| {
                if !(link.span.start as usize <= found.start()
                    && found.end() <= link.span.end as usize)
                {
                    return false;
                }
                let raw = ctx.raw(link.span);
                if !raw.starts_with(['[', '!']) {
                    return true;
                }
                let prefix = if link.image { 2 } else { 1 };
                raw.get(prefix..)
                    .and_then(label_end)
                    .is_some_and(|end| found.start() > link.span.start as usize + prefix + end)
            });
            if destination {
                continue;
            }
            let span = Span::new(found.start() as u32, found.end() as u32);
            if name.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '.' | '-' | '+')) {
                ctx.replace(44, span, span, name);
            } else {
                ctx.report(44, span);
            }
        }
    }
}
