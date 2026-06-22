use std::fmt::Write as _;

use tower_lsp::lsp_types::*;

use crate::document::is_markdown_path;
use crate::frontmatter;
use crate::i18n;
use crate::preview;

use ox_content_mdc_checker::Registry;

use super::assets::{completion_items as asset_completion_items, detect_context, line_prefix};
use super::mdc::{completion_items as mdc_completion_items, detect_site as detect_mdc_site};
use super::snippets::markdown_snippet_items;
use super::Backend;

impl Backend {
    pub(super) async fn completion_response(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<CompletionResponse> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };

        if i18n::is_i18n_source_path(&path) {
            let items = self
                .i18n_state
                .all_dictionary_keys()
                .await
                .into_iter()
                .map(|key| CompletionItem {
                    label: key,
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some("i18n translation key".to_string()),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            return (!items.is_empty()).then_some(CompletionResponse::Array(items));
        }

        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;

        // Asset / link path completion short-circuits everything else
        // when the cursor sits inside a Markdown `()` or an HTML
        // `src=`/`href=`. Mixing it with snippets / frontmatter would
        // pollute the list — a user typing `![alt](./` does not want
        // a `## Section` snippet suggestion.
        let line_text = document.line_text(position.line);
        let prefix = line_prefix(line_text, position.character);
        if let Some((context, partial)) = detect_context(prefix) {
            let doc_dir = path.parent().map(std::path::Path::to_path_buf);
            let src_dir = self.state.root().await;
            let items =
                asset_completion_items(context, partial, doc_dir.as_deref(), src_dir.as_deref());
            return (!items.is_empty()).then_some(CompletionResponse::Array(items));
        }

        let config = self.resolved_config().await;

        // MDC component / attribute completion. Short-circuit out
        // before snippets and frontmatter so the popup is focused on
        // the construct the user is mid-typing — a `## Section`
        // snippet polluting `<Alert |` is just noise.
        let line_text = document.line_text(position.line);
        let prefix = line_prefix(line_text, position.character);
        if let Some(site) = detect_mdc_site(prefix) {
            if let Some(registry) = load_mdc_registry(&config) {
                let items = mdc_completion_items(&site, &registry);
                if !items.is_empty() {
                    return Some(CompletionResponse::Array(items));
                }
            }
        }

        let frontmatter = frontmatter::parse_frontmatter(&document);
        let mut items = frontmatter
            .block
            .as_ref()
            .and_then(|block| {
                Self::load_schema(&config).ok().flatten().and_then(|schema| {
                    frontmatter::completion_items(&document, position, block, &schema)
                })
            })
            .unwrap_or_default();
        items.extend(markdown_snippet_items(&document, position));
        (!items.is_empty()).then_some(CompletionResponse::Array(items))
    }

    pub(super) async fn hover_response(&self, uri: &Url, position: Position) -> Option<Hover> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };

        if i18n::is_i18n_source_path(&path) {
            let path_str = path.to_string_lossy().to_string();
            let usages = self.i18n_state.get_file_key_usages(&path_str).await;
            let key = i18n::key_at_position(&usages, position)?;
            let translations = self.i18n_state.translations_for_key(&key).await;
            if translations.is_empty() {
                return None;
            }

            let mut value = String::new();
            push_fmt(
                &mut value,
                format_args!("**`{key}`**\n\n| Locale | Translation |\n|--------|-------------|\n"),
            );
            for (locale, translation) in &translations {
                push_fmt(&mut value, format_args!("| `{locale}` | {translation} |\n"));
            }

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: None,
            });
        }

        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        let config = self.resolved_config().await;
        let block = frontmatter::parse_frontmatter(&document).block?;
        let schema = Self::load_schema(&config).ok().flatten()?;
        frontmatter::hover(&block, position, &schema)
    }

    pub(super) async fn goto_definition_response(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !i18n::is_i18n_source_path(&path) {
            return None;
        }

        let path_str = path.to_string_lossy().to_string();
        let usages = self.i18n_state.get_file_key_usages(&path_str).await;
        let key = i18n::key_at_position(&usages, position)?;
        let dict_file = self.i18n_state.find_key_definition(&key).await?;
        let target_uri = Url::from_file_path(&dict_file).ok()?;
        let line = i18n::find_key_line_in_file(&dict_file, &key).unwrap_or(0);

        Some(GotoDefinitionResponse::Scalar(Location {
            uri: target_uri,
            range: Range {
                start: Position { line, character: 0 },
                end: Position { line, character: 0 },
            },
        }))
    }

    pub(super) async fn inlay_hints(&self, uri: &Url) -> Option<Vec<InlayHint>> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !i18n::is_i18n_source_path(&path) {
            return None;
        }

        let path_str = path.to_string_lossy().to_string();
        let usages = self.i18n_state.get_file_key_usages(&path_str).await;
        if usages.is_empty() {
            return None;
        }

        let mut hints = Vec::new();
        for usage in &usages {
            if let Some(translation) = self.i18n_state.default_translation(&usage.key).await {
                let label = if translation.len() > 40 {
                    format!(" {}...", &translation[..37])
                } else {
                    format!(" {translation}")
                };

                hints.push(InlayHint {
                    position: Position { line: usage.line - 1, character: usage.end_column - 1 },
                    label: InlayHintLabel::String(label),
                    kind: Some(InlayHintKind::PARAMETER),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                });
            }
        }

        Some(hints)
    }

    pub(super) async fn document_symbols_response(
        &self,
        uri: &Url,
    ) -> Option<DocumentSymbolResponse> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        match preview::document_symbols(document.text(), &document) {
            Ok(symbols) if !symbols.is_empty() => Some(DocumentSymbolResponse::Nested(symbols)),
            _ => None,
        }
    }

    pub(super) async fn folding_range_response(&self, uri: &Url) -> Option<Vec<FoldingRange>> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        let ranges = crate::folding::folding_ranges(&document);
        (!ranges.is_empty()).then_some(ranges)
    }

    pub(super) async fn document_link_response(&self, uri: &Url) -> Option<Vec<DocumentLink>> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if i18n::is_i18n_source_path(&path) {
            let path_str = path.to_string_lossy().to_string();
            let usages = self.i18n_state.get_file_key_usages(&path_str).await;
            let mut links = Vec::new();
            for usage in usages {
                let Some(target) = self.i18n_state.find_key_definition(&usage.key).await else {
                    continue;
                };
                let Some(target) = Url::from_file_path(target).ok() else {
                    continue;
                };
                links.push(DocumentLink {
                    range: Range {
                        start: Position {
                            line: usage.line.saturating_sub(1),
                            character: usage.column.saturating_sub(1),
                        },
                        end: Position {
                            line: usage.line.saturating_sub(1),
                            character: usage.end_column.saturating_sub(1),
                        },
                    },
                    target: Some(target),
                    tooltip: Some("Open translation dictionary".to_string()),
                    data: None,
                });
            }
            return (!links.is_empty()).then_some(links);
        }
        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        let root = self.state.root().await;
        let links = crate::document_link::document_links(&document, &path, root.as_deref());
        (!links.is_empty()).then_some(links)
    }

    pub(super) async fn selection_range_response(
        &self,
        uri: &Url,
        positions: &[Position],
    ) -> Option<Vec<SelectionRange>> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        Some(crate::selection_range::selection_ranges(&document, positions))
    }

    pub(super) async fn document_highlight_response(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<Vec<DocumentHighlight>> {
        let Ok(path) = uri.to_file_path() else {
            return None;
        };
        if !is_markdown_path(&path) {
            return None;
        }

        let document = self.state.document(uri).await?;
        crate::document_highlight::document_highlights(&document, position)
    }
}

fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}

fn load_mdc_registry(config: &crate::config::ResolvedConfig) -> Option<Registry> {
    let path = config.mdc_components.as_deref()?;
    // Treat a missing or unreadable registry file the same as "no
    // registry configured" — completion silently falls through. The
    // alternative (publishing a diagnostic on every keystroke) would
    // be noisy and we'd rather not double the failure modes here.
    Registry::from_path(path).ok().flatten()
}
