use std::{fs, path::Path};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_transform::transformer::parse_frontmatter;
use ox_content_transform::{PublishStateOptions, classify_publish_state};

use crate::{DocumentIndexer, SearchDocument, SearchIndexBuilder};

/// Optional filters applied while walking a content directory.
#[derive(Clone, Debug, Default)]
pub struct SearchIndexBuildOptions {
    /// When set, draft / unlisted / scheduled pages follow publish-state rules.
    pub publish_state: Option<PublishStateOptions>,
    /// Explicit MDX parser override. When omitted, `.mdx` files enable MDX.
    pub mdx: Option<bool>,
}

pub fn extract_search_document_from_source(
    source: &str,
    id: String,
    url: String,
    parser_options: ParserOptions,
) -> SearchDocument {
    let (content, frontmatter) = parse_frontmatter(source);
    let frontmatter_title = frontmatter.get("title").and_then(|v| v.as_str()).map(String::from);
    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    let document = match &result {
        Ok(doc) => {
            let mut indexer = DocumentIndexer::new();
            indexer.extract(doc);

            SearchDocument {
                id,
                title: frontmatter_title
                    .unwrap_or_else(|| indexer.title().map(String::from).unwrap_or_default()),
                url,
                body: indexer.body().to_string(),
                headings: indexer.headings().to_vec(),
                code: indexer.code().to_vec(),
            }
        }
        Err(_) => SearchDocument {
            id,
            title: frontmatter_title.unwrap_or_default(),
            url,
            body: String::new(),
            headings: Vec::new(),
            code: Vec::new(),
        },
    };
    drop(result);

    document
}

pub fn build_search_index_json(documents: impl IntoIterator<Item = SearchDocument>) -> String {
    let mut builder = SearchIndexBuilder::new();

    for doc in documents {
        builder.add_document(doc);
    }

    builder.build().to_json()
}

pub fn search_document_id(src_dir: &Path, file: &str, extensions: &[String]) -> String {
    let file_path = Path::new(file);
    let relative_path = file_path.strip_prefix(src_dir).unwrap_or(file_path);
    let relative_path = relative_path.to_string_lossy().replace('\\', "/");

    crate::strip_markdown_extension(&relative_path, extensions)
}

pub fn build_search_index_from_directory(
    src_dir: &str,
    base: &str,
    extensions: &[String],
) -> String {
    build_search_index_from_directory_with_options(src_dir, base, extensions, None)
}

/// Builds a search index, optionally honoring publish-state frontmatter.
pub fn build_search_index_from_directory_with_options(
    src_dir: &str,
    base: &str,
    extensions: &[String],
    options: Option<&SearchIndexBuildOptions>,
) -> String {
    let src_path = Path::new(src_dir);
    let default_parser_options = ParserOptions::gfm();
    let publish_state = options.and_then(|opts| opts.publish_state.as_ref());
    let documents =
        crate::collect_markdown_files(src_dir, extensions).into_iter().filter_map(|file| {
            let source = fs::read_to_string(&file).ok()?;
            if let Some(publish_state) = publish_state {
                let (_, frontmatter) = parse_frontmatter(&source);
                if !classify_publish_state(&frontmatter, publish_state).listed {
                    return None;
                }
            }
            let id = search_document_id(src_path, &file, extensions);
            let url = format!("{base}{id}");

            let mut parser_options = default_parser_options.clone();
            parser_options.mdx = options.and_then(|opts| opts.mdx).unwrap_or_else(|| {
                Path::new(&file)
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("mdx"))
            });

            Some(extract_search_document_from_source(&source, id, url, parser_options))
        });

    build_search_index_json(documents)
}

#[cfg(test)]
mod tests;
