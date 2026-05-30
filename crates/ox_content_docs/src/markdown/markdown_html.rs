//! HTML rendering (raw-HTML-laced Markdown) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is `MarkdownRenderStyle::Html`
//! (the default). Child module of `markdown`; reuses the parent's
//! extraction/formatting/link helpers via `super::` and emits the ox-content theme
//! HTML structures (`<details>`, stats, member tables, prose blocks, …).

use std::sync::OnceLock;

use super::{
    cached_regex, clean_summary_text, doc_kind_plural, doc_page_href, entry_anchor, fmt_args,
    format_kind_label, generate_source_href, get_entry_badges, member_anchor, normalize_signature,
    parse_example_block, process_doc_text, push_fmt, EntryStats, MarkdownDocsOptions,
    MarkdownLinkContext, MarkdownPathStrategy, RegexCache, DOC_KIND_ORDER,
};
use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc};

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_doc_inline_html(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    render_inline_html(&process_doc_text(text, context))
}

fn render_inline_html(text: &str) -> String {
    static TOKEN_RE: RegexCache = OnceLock::new();

    let Some(token_re) = cached_regex(
        &TOKEN_RE,
        r"`([^`]+)`|\[([^\]]+)\]\(([^)]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|\*([^*]+)\*|_([^_]+)_",
    ) else {
        return escape_html(text).replace('\n', "<br>");
    };
    let mut html = String::new();
    let mut last_index = 0;

    for captures in token_re.captures_iter(text) {
        let Some(mat) = captures.get(0) else {
            continue;
        };
        html.push_str(&escape_html(&text[last_index..mat.start()]));

        if let Some(code) = captures.get(1) {
            push_fmt(&mut html, format_args!("<code>{}</code>", escape_html(code.as_str())));
        } else if let (Some(label), Some(href)) = (captures.get(2), captures.get(3)) {
            push_fmt(
                &mut html,
                format_args!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(href.as_str()),
                    render_inline_html(label.as_str())
                ),
            );
        } else if let Some(strong) = captures.get(4).or_else(|| captures.get(5)) {
            push_fmt(
                &mut html,
                format_args!("<strong>{}</strong>", render_inline_html(strong.as_str())),
            );
        } else if let Some(emphasis) = captures.get(6).or_else(|| captures.get(7)) {
            push_fmt(&mut html, format_args!("<em>{}</em>", render_inline_html(emphasis.as_str())));
        }

        last_index = mat.end();
    }

    html.push_str(&escape_html(&text[last_index..]));
    html.replace('\n', "<br>")
}

fn is_fence_start(line: &str) -> Option<String> {
    static FENCE_RE: RegexCache = OnceLock::new();

    let fence_re = cached_regex(&FENCE_RE, r"^```([\w-]+)?\s*$")?;
    fence_re
        .captures(line.trim())
        .map(|captures| captures.get(1).map_or("text", |value| value.as_str()).to_string())
}

fn heading_match(line: &str) -> Option<(usize, String)> {
    static HEADING_RE: RegexCache = OnceLock::new();

    let heading_re = cached_regex(&HEADING_RE, r"^(#{1,6})\s+(.*)$")?;
    heading_re.captures(line.trim()).map(|captures| {
        (
            captures.get(1).map_or(1, |value| value.as_str().len()).min(6),
            captures.get(2).map_or("", |value| value.as_str()).trim().to_string(),
        )
    })
}

fn ordered_list_item(line: &str) -> Option<String> {
    static ORDERED_RE: RegexCache = OnceLock::new();

    let ordered_re = cached_regex(&ORDERED_RE, r"^\d+\.\s+(.*)$")?;
    ordered_re
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn unordered_list_item(line: &str) -> Option<String> {
    static UNORDERED_RE: RegexCache = OnceLock::new();

    let unordered_re = cached_regex(&UNORDERED_RE, r"^[-*+]\s+(.*)$")?;
    unordered_re
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn is_markdown_block_start(line: &str) -> bool {
    is_fence_start(line).is_some()
        || heading_match(line).is_some()
        || ordered_list_item(line).is_some()
        || unordered_list_item(line).is_some()
}

fn render_markdown_blocks_html(text: &str) -> String {
    static ORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();
    static UNORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();

    let lines: Vec<&str> =
        text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();
    let mut blocks = Vec::new();
    let mut index = 0;
    let ordered_continuation_re = cached_regex(&ORDERED_CONTINUATION_RE, r"^ {0,1}\d+\.\s+");
    let unordered_continuation_re = cached_regex(&UNORDERED_CONTINUATION_RE, r"^[-*+]\s+");

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(language) = is_fence_start(line) {
            let mut code_lines = Vec::new();
            index += 1;

            while index < lines.len() && !lines[index].trim().starts_with("```") {
                code_lines.push(lines[index]);
                index += 1;
            }

            if index < lines.len() {
                index += 1;
            }

            blocks.push(render_code_block_html(&code_lines.join("\n"), &language));
            continue;
        }

        if let Some((level, content)) = heading_match(line) {
            blocks.push(format!("<h{level}>{}</h{level}>", render_inline_html(&content)));
            index += 1;
            continue;
        }

        if let Some(first_item) = ordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| ordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || ordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(format!("<li>{}</li>", render_inline_html(&item_lines.join(" "))));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(format!("<ol>\n{}\n</ol>", items.join("\n")));
            continue;
        }

        if let Some(first_item) = unordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| unordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || unordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(format!("<li>{}</li>", render_inline_html(&item_lines.join(" "))));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(format!("<ul>\n{}\n</ul>", items.join("\n")));
            continue;
        }

        let mut paragraph_lines = vec![trimmed.to_string()];
        index += 1;

        while index < lines.len() {
            let next_line = lines[index];
            let next_trimmed = next_line.trim();

            if next_trimmed.is_empty() || is_markdown_block_start(next_line) {
                break;
            }

            paragraph_lines.push(next_trimmed.to_string());
            index += 1;
        }

        blocks.push(format!("<p>{}</p>", render_inline_html(&paragraph_lines.join(" "))));
    }

    format!("<div class=\"ox-api-entry__prose\">\n{}\n</div>", blocks.join("\n"))
}

fn render_code_block_html(code: &str, language: &str) -> String {
    format!("<pre><code class=\"language-{language}\">{}</code></pre>", escape_html(code))
}

fn render_highlighted_inline_code_html(code: &str, class_name: &str, language: &str) -> String {
    format!(
        "<code class=\"{} language-{language}\">{}</code>",
        escape_html(class_name),
        escape_html(code)
    )
}

pub(super) fn render_details_controls_html(target_selector: &str) -> String {
    format!(
        "<div class=\"ox-api-controls\" data-ox-api-target=\"{target_selector}\" role=\"toolbar\" aria-label=\"Reference display controls\">
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"expand\">Open all</button>
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"collapse\">Close all</button>
</div>"
    )
}

pub(super) fn render_stats_html(stats: &EntryStats, module_count: Option<usize>) -> String {
    let mut items = Vec::new();

    if let Some(module_count) = module_count {
        items.push(("modules".to_string(), module_count, None));
    }

    items.push(("symbols".to_string(), stats.entries, None));

    for kind in DOC_KIND_ORDER {
        if let Some(count) = stats.by_kind.get(kind).copied().filter(|count| *count > 0) {
            items.push((doc_kind_plural(kind).to_string(), count, None));
        }
    }

    if stats.params > 0 {
        items.push(("parameters".to_string(), stats.params, None));
    }
    if stats.members > 0 {
        items.push(("members".to_string(), stats.members, None));
    }
    if stats.returns > 0 {
        items.push(("returns".to_string(), stats.returns, None));
    }
    if stats.examples > 0 {
        items.push(("examples".to_string(), stats.examples, None));
    }
    if stats.deprecated > 0 {
        items.push(("deprecated".to_string(), stats.deprecated, Some("warning")));
    }

    let rendered_items = items
        .into_iter()
        .map(|(label, value, tone)| {
            format!(
                "<span class=\"ox-api-stat{}\">
  <strong>{value}</strong>
  <span>{}</span>
</span>",
                tone.map_or(String::new(), |tone| format!(" ox-api-stat--{tone}")),
                escape_html(&label)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("<div class=\"ox-api-stats\" aria-label=\"API reference summary\">\n{rendered_items}\n</div>")
}

fn render_entry_badges_html(entry: &ApiDocEntry, class_name: &str) -> String {
    let badges = get_entry_badges(entry);
    if badges.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    for badge in badges {
        let tone_class = badge
            .tone
            .map_or(String::new(), |tone| fmt_args(format_args!(" ox-api-badge--{tone}")));
        push_fmt(
            &mut rendered,
            format_args!(
                "<span class=\"ox-api-badge{}\">{}</span>",
                tone_class,
                escape_html(&badge.label)
            ),
        );
    }

    fmt_args(format_args!("<span class=\"{class_name}\">{rendered}</span>"))
}

fn render_overview_html_item(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&process_doc_text(&entry.description, context), 88);
    let meta = render_entry_badges_html(entry, "ox-api-module__meta");
    let heading = if let Some(signature) = signature {
        format!(
            "<a href=\"{}\" class=\"ox-api-module__link\">{}</a>",
            escape_html(href),
            render_highlighted_inline_code_html(
                &signature,
                "ox-api-module__signature ox-api-module__signature--highlighted",
                "typescript",
            )
        )
    } else {
        format!(
            "<a href=\"{}\" class=\"ox-api-module__link\"><code class=\"ox-api-module__name\">{}</code></a>",
            escape_html(href),
            escape_html(&entry.name)
        )
    };

    format!(
        "<li><span class=\"ox-api-module__kind\">{}</span><div class=\"ox-api-module__item\">{}{summary_html}{meta}</div></li>",
        escape_html(format_kind_label(&entry.kind)),
        heading,
        summary_html = if summary.is_empty() {
            String::new()
        } else {
            format!("<span class=\"ox-api-module__summary\">{}</span>", render_inline_html(&summary))
        }
    )
}

fn render_params_list_html(
    params: &[ApiParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let rows = params
        .iter()
        .map(|param| {
            let mut flags = Vec::new();
            if param.optional {
                flags.push("optional".to_string());
            }
            if let Some(default_value) = &param.default_value {
                flags.push(format!("default: {default_value}"));
            }
            let flag_text = flags.join(" · ");
            let description = [param.description.as_str(), flag_text.as_str()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" — ");

            format!(
                "<li class=\"ox-api-entry__param\">
  <div class=\"ox-api-entry__param-heading\">
    <code class=\"ox-api-entry__param-name\">{}</code>
    <code class=\"ox-api-entry__param-type\">{}</code>
  </div>
  {}
</li>",
                escape_html(&param.name),
                escape_html(&param.type_annotation),
                if description.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p class=\"ox-api-entry__param-description\">{}</p>",
                        render_doc_inline_html(&description, context)
                    )
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<ul class=\"ox-api-entry__params\">
{rows}
</ul>
</div>"
    )
}

fn render_tag_list_html(tags: &[ApiDocTag], context: Option<&MarkdownLinkContext<'_>>) -> String {
    let mut items = String::new();
    for tag in tags {
        push_fmt(&mut items, format_args!(
            "<li><span class=\"ox-api-entry__tag-name\">@{}</span><span class=\"ox-api-entry__tag-value\">{}</span></li>",
            escape_html(&tag.tag),
            render_doc_inline_html(&tag.value, context)
        ));
    }

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--tags\">
<h4>Tags</h4>
<ul class=\"ox-api-entry__tags\">{items}</ul>
</div>"
    )
}

fn render_member_flags(member: &ApiDocMember) -> String {
    let mut flags = Vec::new();
    if member.optional {
        flags.push("optional");
    }
    if member.readonly {
        flags.push("readonly");
    }
    if member.r#static {
        flags.push("static");
    }
    if member.private {
        flags.push("private");
    }

    let mut html = String::new();
    for flag in flags {
        push_fmt(&mut html, format_args!("<span class=\"ox-api-badge\">{flag}</span>"));
    }
    html
}

fn render_member_type_html(member: &ApiDocMember) -> String {
    let value = member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()));

    value.map_or_else(String::new, |value| {
        render_highlighted_inline_code_html(value, "ox-api-entry__member-type", "typescript")
    })
}

fn render_member_description_html(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut blocks = Vec::new();

    if !member.description.is_empty() {
        blocks.push(format!(
            "<div class=\"ox-api-entry__member-description\">{}</div>",
            render_doc_inline_html(&member.description, context)
        ));
    }

    if !member.params.is_empty() {
        let mut params = String::new();
        for param in &member.params {
            let mut description = param.description.clone();
            if param.optional {
                if description.is_empty() {
                    description.push_str("optional");
                } else {
                    description.push_str(" - optional");
                }
            }
            push_fmt(
                &mut params,
                format_args!(
                    "<li><code>{}</code>{}</li>",
                    escape_html(&param.name),
                    if description.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", render_doc_inline_html(&description, context))
                    }
                ),
            );
        }
        blocks.push(format!("<ul class=\"ox-api-entry__member-params\">{params}</ul>"));
    }

    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            blocks.push(format!(
                "<div class=\"ox-api-entry__member-return\"><span>Returns</span> {}</div>",
                render_doc_inline_html(&returns.description, context)
            ));
        }
    }

    blocks.join("")
}

fn render_member_table_html(
    entry_name: &str,
    title: &str,
    members: &[&ApiDocMember],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let rows = members
        .iter()
        .map(|member| {
            format!(
                "<tr id=\"{}\">
  <td><code>{}</code>{}</td>
  <td><span class=\"ox-api-entry__member-kind\">{}</span></td>
  <td>{}</td>
  <td>{}</td>
</tr>",
                escape_html(&member_anchor(
                    entry_name,
                    member,
                    context.map_or(MarkdownPathStrategy::Flat, |context| context
                        .options
                        .path_strategy),
                )),
                escape_html(&member.name),
                render_member_flags(member),
                escape_html(&member.kind),
                render_member_type_html(member),
                render_member_description_html(member, context)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<div class=\"ox-api-entry__member-group\">
<h5>{}</h5>
<table class=\"ox-api-entry__members-table\">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
</div>",
        escape_html(title)
    )
}

fn render_members_table_html(
    entry: &ApiDocEntry,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if entry.members.is_empty() {
        return String::new();
    }

    let constructors =
        entry.members.iter().filter(|member| member.kind == "constructor").collect::<Vec<_>>();
    let static_methods = entry
        .members
        .iter()
        .filter(|member| {
            member.r#static && matches!(member.kind.as_str(), "method" | "getter" | "setter")
        })
        .collect::<Vec<_>>();
    let methods = entry
        .members
        .iter()
        .filter(|member| {
            !member.r#static && matches!(member.kind.as_str(), "method" | "getter" | "setter")
        })
        .collect::<Vec<_>>();
    let static_properties = entry
        .members
        .iter()
        .filter(|member| member.r#static && member.kind == "property")
        .collect::<Vec<_>>();
    let properties = entry
        .members
        .iter()
        .filter(|member| !member.r#static && member.kind == "property")
        .collect::<Vec<_>>();
    let enum_members =
        entry.members.iter().filter(|member| member.kind == "enumMember").collect::<Vec<_>>();

    let mut groups = Vec::new();
    match entry.kind.as_str() {
        "class" => {
            groups.push(render_member_table_html(
                &entry.name,
                "Constructors",
                &constructors,
                context,
            ));
            groups.push(render_member_table_html(
                &entry.name,
                "Static Methods",
                &static_methods,
                context,
            ));
            groups.push(render_member_table_html(&entry.name, "Methods", &methods, context));
            groups.push(render_member_table_html(
                &entry.name,
                "Static Properties",
                &static_properties,
                context,
            ));
            groups.push(render_member_table_html(&entry.name, "Properties", &properties, context));
        }
        "interface" => {
            groups.push(render_member_table_html(&entry.name, "Properties", &properties, context));
            groups.push(render_member_table_html(&entry.name, "Methods", &methods, context));
        }
        "type" => {
            groups.push(render_member_table_html(&entry.name, "Properties", &properties, context));
            groups.push(render_member_table_html(&entry.name, "Methods", &methods, context));
            groups.push(render_member_table_html(
                &entry.name,
                "Enum Members",
                &enum_members,
                context,
            ));
        }
        _ => groups.push(render_member_table_html(
            &entry.name,
            "Members",
            &entry.members.iter().collect::<Vec<_>>(),
            context,
        )),
    }

    let groups = groups.into_iter().filter(|group| !group.is_empty()).collect::<Vec<_>>();
    if groups.is_empty() {
        return String::new();
    }

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--members\">
<h4>Members</h4>
{}
</div>",
        groups.join("\n")
    )
}

fn render_entry_body_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    let source_href = options.github_url.as_ref().map(|github_url| {
        generate_source_href(&entry.file, github_url, Some(entry.line), Some(entry.end_line))
    });
    let mut body = String::new();

    if !processed_description.is_empty() {
        body.push_str(&render_markdown_blocks_html(&processed_description));
        body.push('\n');
    }

    if let Some(signature) = &entry.signature {
        push_fmt(
            &mut body,
            format_args!(
                "<div class=\"ox-api-entry__section ox-api-entry__section--signature\">
<h4>Signature</h4>
{}
</div>\n",
                render_code_block_html(signature, "typescript")
            ),
        );
    }

    if let Some(source_href) = source_href {
        push_fmt(&mut body, format_args!(
            "<p class=\"ox-api-entry__source\"><a class=\"ox-api-entry__source-link\" href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">View source<span class=\"ox-api-entry__source-icon\" aria-hidden=\"true\"></span></a></p>\n",
            escape_html(&source_href)
        ));
    }

    if !entry.members.is_empty() {
        body.push_str(&render_members_table_html(entry, link_context));
        body.push('\n');
    }

    if !entry.params.is_empty() {
        body.push_str(&render_params_list_html(&entry.params, link_context));
        body.push('\n');
    }

    if let Some(returns) = &entry.returns {
        push_fmt(
            &mut body,
            format_args!(
                "<div class=\"ox-api-entry__section ox-api-entry__section--returns\">
<h4>Returns</h4>
<div class=\"ox-api-entry__return\">
  <code class=\"ox-api-entry__return-type\">{}</code>
  {}
</div>
</div>\n",
                escape_html(&returns.type_annotation),
                if returns.description.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p class=\"ox-api-entry__return-description\">{}</p>",
                        render_doc_inline_html(&returns.description, link_context)
                    )
                }
            ),
        );
    }

    if !entry.examples.is_empty() {
        let examples_html = entry
            .examples
            .iter()
            .enumerate()
            .map(|(index, example)| {
                let (code, language) = parse_example_block(example);
                format!(
                    "<div class=\"ox-api-entry__example\">
<div class=\"ox-api-entry__example-heading\">Example {}</div>
{}
</div>",
                    index + 1,
                    render_code_block_html(&code, &language)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        push_fmt(
            &mut body,
            format_args!(
                "<div class=\"ox-api-entry__section ox-api-entry__section--examples\">
<h4>Examples</h4>
{examples_html}
</div>\n"
            ),
        );
    }

    if !entry.tags.is_empty() {
        body.push_str(&render_tag_list_html(&entry.tags, link_context));
        body.push('\n');
    }

    body.trim().to_string()
}

pub(super) fn render_entry_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    let summary_signature = normalize_signature(entry.signature.as_deref());
    let body = render_entry_body_html(entry, options, link_context);

    let summary_description = clean_summary_text(
        &processed_description,
        if summary_signature.is_some() { 80 } else { 120 },
    );
    let summary_heading = if let Some(summary_signature) = summary_signature {
        render_highlighted_inline_code_html(
            &summary_signature,
            "ox-api-entry__signature ox-api-entry__signature--highlighted",
            "typescript",
        )
    } else {
        format!("<code class=\"ox-api-entry__name\">{}</code>", escape_html(&entry.name))
    };
    let summary_parts = [
        format!(
            "<span class=\"ox-api-entry__kind\">{}</span>",
            escape_html(format_kind_label(&entry.kind))
        ),
        format!(
            "<span class=\"ox-api-entry__summary-main\">{}{}{}</span>",
            summary_heading,
            if summary_description.is_empty() {
                String::new()
            } else {
                format!(
                    "<span class=\"ox-api-entry__description\">{}</span>",
                    render_inline_html(&summary_description)
                )
            },
            render_entry_badges_html(entry, "ox-api-entry__meta")
        ),
    ];

    format!(
        "<details id=\"{}\" class=\"ox-api-entry\">
  <summary>{}</summary>
  <div class=\"ox-api-entry__body\">
{}
  </div>
</details>

",
        entry_anchor(&entry.name),
        summary_parts.join(""),
        body
    )
}

pub(super) fn render_entry_page_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let body = render_entry_body_html(entry, options, link_context);
    fmt_args(format_args!(
        "<div id=\"{}\" class=\"ox-api-entry ox-api-entry--page\">
{}
</div>
",
        entry_anchor(&entry.name),
        body
    ))
}

pub(super) fn render_module_section_html(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    file_name: &str,
    display_name: &str,
    count_label: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut markdown = fmt_args(format_args!(
        "<details class=\"ox-api-module\">
  <summary>
    <span class=\"ox-api-module__title\"><a href=\"{}\">{}</a></span>
    <span class=\"ox-api-module__count\">{count_label}</span>
  </summary>
  <div class=\"ox-api-module__body\">
    <ul class=\"ox-api-module__list\">
",
        escape_html(&doc_page_href(options, file_name, None)),
        escape_html(display_name)
    ));

    for entry in &doc.entries {
        let href = doc_page_href(options, file_name, Some(&entry_anchor(&entry.name)));
        push_fmt(
            &mut markdown,
            format_args!("      {}\n", render_overview_html_item(entry, &href, link_context)),
        );
    }

    markdown.push_str(
        "    </ul>
  </div>
</details>

",
    );

    markdown
}
