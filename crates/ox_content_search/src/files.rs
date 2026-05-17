//! File discovery helpers for search indexing.

use std::fs;
use std::path::Path;

/// Collects Markdown files under `src_dir` for search indexing.
#[must_use]
pub fn collect_markdown_files(src_dir: &str, extensions: &[String]) -> Vec<String> {
    let extensions = normalize_markdown_extensions(extensions);
    let mut files = Vec::new();
    collect_markdown_files_inner(Path::new(src_dir), &extensions, &mut files);
    files.sort();
    files
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
}
