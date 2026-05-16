use tower_lsp::lsp_types::*;

use crate::document::is_markdown_path;
use crate::frontmatter;
use crate::i18n;
use crate::preview;

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
        let config = self.resolved_config().await;
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

            let mut value =
                format!("**`{key}`**\n\n| Locale | Translation |\n|--------|-------------|\n");
            for (locale, translation) in &translations {
                value.push_str(&format!("| `{locale}` | {translation} |\n"));
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
}
