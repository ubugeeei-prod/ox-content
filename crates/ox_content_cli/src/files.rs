use crate::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{Component, Path, PathBuf},
};
use walkdir::WalkDir;

pub const MARKDOWN_GLOB: &str = "**/*.{md,markdown,mdx,mdc}";
pub const VIEWER_LIMIT: u64 = 4 * 1024 * 1024;

pub fn absolute(path: &Path) -> Result<PathBuf> {
    let path = std::path::absolute(path)?;
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(part),
        }
    }
    Ok(result)
}

pub fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn glob(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}

pub fn discover(paths: &[String], ignore: &[String]) -> Result<Vec<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let ignored = glob(ignore)?;
    let defaults = vec![".".to_string()];
    let mut found = BTreeSet::new();
    for input in if paths.is_empty() { &defaults } else { paths } {
        let path = absolute(Path::new(input))?;
        let pattern = if path.is_dir() { path.join(MARKDOWN_GLOB) } else { path.clone() };
        let matcher = glob(&[slash(&pattern)])?;
        let mut root = PathBuf::new();
        for component in pattern.components() {
            if component.as_os_str().to_string_lossy().contains(['*', '?', '[', '{']) {
                break;
            }
            root.push(component);
        }
        if !root.is_dir() {
            root.pop();
        }
        for entry in WalkDir::new(&root).follow_links(false).into_iter().filter_entry(|entry| {
            !["node_modules", ".git", "dist", "target"]
                .iter()
                .any(|name| entry.file_name() == *name)
                && !ignored.is_match(entry.path())
                && !ignored
                    .is_match(entry.path().strip_prefix(&cwd).unwrap_or_else(|_| entry.path()))
        }) {
            let entry = entry?;
            if entry.file_type().is_file() && matcher.is_match(entry.path()) {
                found.insert(entry.path().to_path_buf());
            }
        }
    }
    Ok(found.into_iter().collect())
}

pub fn read_document(path: &Path) -> Result<String> {
    if std::fs::metadata(path)?.len() > VIEWER_LIMIT {
        return Err("Document exceeds the 4 MiB viewer limit".into());
    }
    read_bounded(std::fs::File::open(path)?)
}

pub fn read_bounded(reader: impl Read) -> Result<String> {
    let mut bytes = Vec::new();
    reader.take(VIEWER_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > VIEWER_LIMIT {
        return Err("Document exceeds the 4 MiB viewer limit".into());
    }
    Ok(String::from_utf8(bytes)?)
}
