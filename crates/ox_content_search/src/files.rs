//! File discovery helpers for search indexing.

use std::fs;
use std::path::{Path, PathBuf};

/// Collects Markdown files under `src_dir` for search indexing.
#[must_use]
pub fn collect_markdown_files(src_dir: &str, extensions: &[String]) -> Vec<String> {
    let extensions = normalize_markdown_extensions(extensions);
    let mut files = Vec::new();
    collect_markdown_files_inner(Path::new(src_dir), &extensions, &mut files);
    files.sort();
    files
}

/// Removes the matching Markdown extension from a file path.
#[must_use]
pub fn strip_markdown_extension(file_path: &str, extensions: &[String]) -> String {
    let mut extensions = normalize_markdown_extensions(extensions);
    extensions.sort_by_key(|extension| std::cmp::Reverse(extension.len()));

    let lower_path = file_path.to_ascii_lowercase();
    extensions.iter().find(|extension| lower_path.ends_with(extension.as_str())).map_or_else(
        || file_path.to_string(),
        |extension| file_path[..file_path.len() - extension.len()].to_string(),
    )
}

/// Writes a serialized search index to `search-index.json` under `out_dir`.
pub fn write_search_index(index_json: &str, out_dir: &str) -> std::io::Result<PathBuf> {
    let out_dir = Path::new(out_dir);
    fs::create_dir_all(out_dir)?;

    let index_path = out_dir.join("search-index.json");
    fs::write(&index_path, index_json)?;

    Ok(index_path)
}

fn collect_markdown_files_inner(dir: &Path, extensions: &[String], files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "node_modules" {
                continue;
            }
            collect_markdown_files_inner(&path, extensions, files);
        } else if file_type.is_file() && is_markdown_file(&path, extensions) {
            files.push(path.to_string_lossy().into_owned());
        }
    }
}

fn normalize_markdown_extensions(extensions: &[String]) -> Vec<String> {
    let source = if extensions.is_empty() {
        vec![".md".to_string(), ".markdown".to_string(), ".mdx".to_string()]
    } else {
        extensions.to_vec()
    };

    source
        .into_iter()
        .map(|extension| {
            let extension =
                if extension.starts_with('.') { extension } else { format!(".{extension}") };
            extension.to_ascii_lowercase()
        })
        .collect()
}

fn is_markdown_file(path: &Path, extensions: &[String]) -> bool {
    let path = path.to_string_lossy().to_ascii_lowercase();
    extensions.iter().any(|extension| path.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_markdown_files_for_search() {
        let root =
            std::env::temp_dir().join(format!("ox-content-search-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("docs/guide")).unwrap();
        std::fs::create_dir_all(root.join("docs/.drafts")).unwrap();
        std::fs::create_dir_all(root.join("docs/node_modules/pkg")).unwrap();
        std::fs::write(root.join("docs/index.md"), "").unwrap();
        std::fs::write(root.join("docs/guide/intro.mdx"), "").unwrap();
        std::fs::write(root.join("docs/.drafts/hidden.md"), "").unwrap();
        std::fs::write(root.join("docs/node_modules/pkg/readme.md"), "").unwrap();

        let files = collect_markdown_files(
            root.join("docs").to_string_lossy().as_ref(),
            &["md".to_string(), ".mdx".to_string()],
        );

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| file.ends_with("docs/index.md")));
        assert!(files.iter().any(|file| file.ends_with("docs/guide/intro.mdx")));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn strips_longest_matching_markdown_extension() {
        assert_eq!(
            strip_markdown_extension(
                "guide/reference.markdown",
                &[".md".to_string(), ".markdown".to_string()]
            ),
            "guide/reference"
        );
        assert_eq!(
            strip_markdown_extension("guide/reference.MDX", &["mdx".to_string()]),
            "guide/reference"
        );
        assert_eq!(
            strip_markdown_extension("guide/reference.txt", &["md".to_string()]),
            "guide/reference.txt"
        );
    }

    #[test]
    fn writes_search_index_file() {
        let root =
            std::env::temp_dir().join(format!("ox-content-search-index-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        let index_path =
            write_search_index(r#"{"doc_count":0}"#, root.to_string_lossy().as_ref()).unwrap();

        assert_eq!(index_path.file_name().unwrap(), "search-index.json");
        assert_eq!(std::fs::read_to_string(&index_path).unwrap(), r#"{"doc_count":0}"#);

        let _ = std::fs::remove_dir_all(&root);
    }
}
