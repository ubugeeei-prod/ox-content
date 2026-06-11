use super::*;

fn render_entry_body_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    // Entries with an empty `file` (e.g. symbols re-exported from an external
    // package) have no source in the consumer's repo, so emit no source link.
    let source_href =
        options.github_url.as_ref().filter(|_| !entry.file.is_empty()).map(|github_url| {
            generate_source_href(&entry.file, github_url, Some(entry.line), Some(entry.end_line))
        });
    let mut body = String::new();

    if !processed_description.is_empty() {
        body.push_str(&render_markdown_blocks_html(&processed_description));
        body.push('\n');
    }
    push_heritage_sections_html(&mut body, entry, link_context);

    if let Some(signature) = &entry.signature {
        body.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--signature\">
<h4>Signature</h4>
",
        );
        body.push_str(&render_code_block_html(signature, "typescript"));
        body.push_str(
            "
</div>\n",
        );
    }

    if let Some(source_href) = source_href {
        body.push_str(
            "<p class=\"ox-api-entry__source\"><a class=\"ox-api-entry__source-link\" href=\"",
        );
        body.push_str(&escape_html(&source_href));
        body.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">View source<span class=\"ox-api-entry__source-icon\" aria-hidden=\"true\"></span></a></p>\n");
    }

    push_type_parameters_html(&mut body, &entry.type_parameters, options, link_context);

    if !entry.members.is_empty() {
        body.push_str(&render_members_html(entry, options, link_context));
        body.push('\n');
    }

    push_params_html(&mut body, &entry.params, options, link_context);

    if let Some(returns) = &entry.returns {
        push_returns_html(&mut body, returns, options, link_context);
    }

    push_throws_html(&mut body, &entry.throws, &entry.tags, link_context);

    push_examples_html(&mut body, &entry.examples);

    push_tag_list_html(&mut body, &entry.tags, link_context);

    body.trim().to_string()
}

pub fn render_entry_html(
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
        join3("<code class=\"ox-api-entry__name\">", &escape_html(&entry.name), "</code>")
    };
    let summary_description = if summary_description.is_empty() {
        String::new()
    } else {
        join3(
            "<span class=\"ox-api-entry__description\">",
            &render_inline_html(&summary_description),
            "</span>",
        )
    };
    let badges = render_entry_badges_html(entry, "ox-api-entry__meta");
    let kind = escape_html(format_kind_label(&entry.kind));
    let mut summary = StringBuilder::with_capacity(
        kind.len() + summary_heading.len() + summary_description.len() + badges.len() + 92,
    );
    summary.push_str("<span class=\"ox-api-entry__kind\">");
    summary.push_str(&kind);
    summary.push_str("</span><span class=\"ox-api-entry__summary-main\">");
    summary.push_str(&summary_heading);
    summary.push_str(&summary_description);
    summary.push_str(&badges);
    summary.push_str("</span>");
    let summary = summary.into_string();
    let anchor = entry_anchor(&entry.name);

    let mut out = StringBuilder::with_capacity(anchor.len() + summary.len() + body.len() + 120);
    out.push_str("<details id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry\">
  <summary>",
    );
    out.push_str(&summary);
    out.push_str(
        "</summary>
  <div class=\"ox-api-entry__body\">
",
    );
    out.push_str(&body);
    out.push_str(
        "
  </div>
</details>

",
    );
    out.into_string()
}

pub fn render_entry_page_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let body = render_entry_body_html(entry, options, link_context);
    // A per-symbol page has no `<summary>`, so structured tags (lifecycle / since)
    // would otherwise be invisible once excluded from the generic tag list. Surface
    // them as a badge row at the top of the page instead.
    let badges = render_entry_badges_html(entry, "ox-api-entry__meta");
    let anchor = entry_anchor(&entry.name);
    let mut out = StringBuilder::with_capacity(anchor.len() + badges.len() + body.len() + 80);
    out.push_str("<div id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry ox-api-entry--page\">
",
    );
    if !badges.is_empty() {
        out.push_str(&badges);
        out.push_char('\n');
    }
    out.push_str(&body);
    out.push_str(
        "
</div>
",
    );
    out.into_string()
}

/// Renders an overloaded function's symbol page body in HTML: a symbol-level badge
/// row + comment hoisted from the implementation, then one `Call Signature` section
/// per public overload. The implementation signature is omitted (TypeDoc parity).
pub fn render_overload_body_html(
    public: &[&ApiDocEntry],
    implementation: Option<&ApiDocEntry>,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    // Symbol-level badges/comment come from the implementation when present;
    // otherwise fall back to the first public signature.
    let symbol = implementation.or_else(|| public.first().copied());
    let anchor = symbol.map(|entry| entry_anchor(&entry.name)).unwrap_or_default();

    let mut out = String::new();
    out.push_str("<div id=\"");
    out.push_str(&anchor);
    out.push_str("\" class=\"ox-api-entry ox-api-entry--page\">\n");

    if let Some(symbol) = symbol {
        let badges = render_entry_badges_html(symbol, "ox-api-entry__meta");
        if !badges.is_empty() {
            out.push_str(&badges);
            out.push('\n');
        }
    }
    if let Some(implementation) = implementation {
        let description = process_doc_text(&implementation.description, link_context);
        if !description.is_empty() {
            out.push_str(&render_markdown_blocks_html(&description));
            out.push('\n');
        }
    }

    for signature in public {
        out.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--call-signature\">
<h4>Call Signature</h4>
",
        );
        if let Some(code) = &signature.signature {
            out.push_str(&render_code_block_html(code, "typescript"));
            out.push('\n');
        }
        let description = process_doc_text(&signature.description, link_context);
        if !description.is_empty() {
            out.push_str(&render_markdown_blocks_html(&description));
            out.push('\n');
        }
        push_type_parameters_html(&mut out, &signature.type_parameters, options, link_context);
        push_params_html(&mut out, &signature.params, options, link_context);
        if let Some(returns) = &signature.returns {
            push_returns_html(&mut out, returns, options, link_context);
        }
        push_throws_html(&mut out, &signature.throws, &signature.tags, link_context);
        push_examples_html(&mut out, &signature.examples);
        push_tag_list_html(&mut out, &signature.tags, link_context);
        out.push_str("</div>\n");
    }

    out.push_str("</div>\n");
    out
}
