use std::{fs, path::Path};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_transform::transformer::parse_frontmatter;

use crate::{DocumentIndexer, SearchDocument, SearchIndexBuilder};

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

fn should_omit_from_search(id: &str, source: &str) -> bool {
    let stem = id.rsplit('/').next().unwrap_or(id);
    if stem.eq_ignore_ascii_case("404") {
        return true;
    }
    let (_, frontmatter) = parse_frontmatter(source);
    frontmatter_flag(frontmatter.get("noindex")) || frontmatter_flag(frontmatter.get("draft"))
}

fn frontmatter_flag(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(true)) => true,
        Some(serde_json::Value::String(value)) if value.eq_ignore_ascii_case("true") => true,
        _ => false,
    }
}

pub fn build_search_index_from_directory(
    src_dir: &str,
    base: &str,
    extensions: &[String],
) -> String {
    let src_path = Path::new(src_dir);
    let parser_options = ParserOptions::gfm();
    let documents =
        crate::collect_markdown_files(src_dir, extensions).into_iter().filter_map(|file| {
            let source = fs::read_to_string(&file).ok()?;
            let id = search_document_id(src_path, &file, extensions);
            if should_omit_from_search(&id, &source) {
                return None;
            }
            let url = format!("{base}{id}");

            Some(extract_search_document_from_source(&source, id, url, parser_options.clone()))
        });

    build_search_index_json(documents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omits_404_and_noindex_documents() {
        let root = std::env::temp_dir()
            .join(format!("ox-content-search-not-found-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("guide.md"), "---\ntitle: Guide\n---\n\nPublished.\n").unwrap();
        std::fs::write(root.join("404.md"), "---\ntitle: Lost\n---\n\nNot indexed.\n").unwrap();
        std::fs::write(
            root.join("secret.md"),
            "---\ntitle: Secret\nnoindex: true\n---\n\nHidden.\n",
        )
        .unwrap();

        let index = build_search_index_from_directory(
            root.to_string_lossy().as_ref(),
            "/",
            &[".md".to_string()],
        );
        let _ = std::fs::remove_dir_all(&root);

        assert!(index.contains("Guide"), "{index}");
        assert!(!index.contains("Lost"), "{index}");
        assert!(!index.contains("Secret"), "{index}");
    }
}
