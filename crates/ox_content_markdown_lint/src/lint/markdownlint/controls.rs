use super::{
    Context, catalog,
    config::{MarkdownlintConfig, Settings},
};
use std::borrow::Cow;

struct Directive<'s> {
    line: usize,
    action: &'static str,
    arguments: &'s str,
}

pub(super) fn run(ctx: &mut Context<'_, '_>) {
    if !ctx.settings.inline_config {
        super::rules(ctx);
        return;
    }
    let mut directives = Vec::new();
    let mut offset = crate::lint::source::frontmatter_end(ctx.source.text);
    while let Some(start) = ctx.source.text[offset..].find("<!--") {
        let start = offset + start;
        let Some(end) = ctx.source.text[start + 4..].find("-->") else {
            break;
        };
        let end = start + 4 + end;
        let content = ctx.source.text[start + 4..end].trim();
        if content.get(..13).is_some_and(|prefix| prefix.eq_ignore_ascii_case("markdownlint-")) {
            let content = &content[13..];
            let (action, arguments) =
                content.split_once(char::is_whitespace).unwrap_or((content, ""));
            let Some(action) = [
                "configure-file",
                "disable-file",
                "enable-file",
                "capture",
                "restore",
                "disable",
                "enable",
                "disable-line",
                "disable-next-line",
            ]
            .into_iter()
            .find(|name| name.eq_ignore_ascii_case(action)) else {
                offset = end + 3;
                continue;
            };
            directives.push(Directive {
                line: ctx.source.line_index(start) + 1,
                action,
                arguments: arguments.trim(),
            });
        }
        offset = end + 3;
    }
    if directives.is_empty() {
        super::rules(ctx);
        return;
    }
    let mut settings = Cow::Borrowed(ctx.settings);
    for directive in directives.iter().filter(|d| d.action == "configure-file") {
        if let Ok(config) = serde_json::from_str::<MarkdownlintConfig>(directive.arguments) {
            let mut effective = settings.configuration.0.as_object().cloned().unwrap_or_default();
            if let Some(object) = config.0.as_object() {
                effective.extend(object.clone());
            }
            settings = Cow::Owned(Settings::new(&MarkdownlintConfig(effective.into())));
        }
    }
    let mut enabled = settings.enabled;
    for directive in
        directives.iter().filter(|d| matches!(d.action, "disable-file" | "enable-file"))
    {
        apply(&mut enabled, directive);
    }
    let mut captured = enabled;
    let mut changes = vec![(1, enabled)];
    let mut overrides = Vec::new();
    let mut possible = enabled;
    for directive in &directives {
        match directive.action {
            "capture" => captured = enabled,
            "restore" => enabled = captured,
            "disable" | "enable" => apply(&mut enabled, directive),
            "disable-line" | "disable-next-line" => {
                let line = directive.line + usize::from(directive.action == "disable-next-line");
                overrides.push((line, mask(directive.arguments)));
            }
            _ => continue,
        }
        possible |= enabled;
        changes.push((directive.line, enabled));
    }
    if possible != settings.enabled {
        settings.to_mut().enabled = possible;
    }
    let start = ctx.output.len();
    let mut effective = Context {
        settings: &settings,
        source: ctx.source,
        document: ctx.document,
        output: ctx.output,
    };
    super::rules(&mut effective);
    let mut kept = start;
    for index in start..ctx.output.len() {
        let value = &ctx.output[index];
        let bit = catalog::mask(&value.rule_id);
        let line = value.line as usize;
        let state = changes.partition_point(|&(at, _)| at <= line).saturating_sub(1);
        let enabled = changes[state].1 & bit != 0;
        let disabled = overrides.iter().any(|&(at, bits)| at == line && bits & bit != 0);
        if enabled && !disabled {
            ctx.output.swap(kept, index);
            kept += 1;
        }
    }
    ctx.output.truncate(kept);
}

fn mask(arguments: &str) -> u64 {
    if arguments.is_empty() {
        catalog::all()
    } else {
        arguments.split_whitespace().fold(0, |bits, name| bits | catalog::mask(name))
    }
}
fn apply(state: &mut u64, directive: &Directive<'_>) {
    let bits = mask(directive.arguments);
    if directive.action.starts_with("enable") {
        *state |= bits;
    } else {
        *state &= !bits;
    }
}
