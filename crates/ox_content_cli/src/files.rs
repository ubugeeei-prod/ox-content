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

pub fn relative(path: &Path, base: &Path) -> PathBuf {
    let mut path_parts = path.components().peekable();
    let mut base_parts = base.components().peekable();
    let mut common = false;
    while path_parts.peek().is_some() && path_parts.peek() == base_parts.peek() {
        common = true;
        path_parts.next();
        base_parts.next();
    }
    if !common {
        return path.to_path_buf();
    }
    let mut result = PathBuf::new();
    for _ in base_parts {
        result.push("..");
    }
    for part in path_parts {
        result.push(part);
    }
    result
}

fn glob(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}

pub fn discover(paths: &[String], ignore: &[String]) -> Result<Vec<PathBuf>> {
    discover_with_ignore_file(paths, ignore, Path::new(""))
}

pub fn discover_with_ignore_file(
    paths: &[String],
    ignore: &[String],
    ignore_file: &Path,
) -> Result<Vec<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let ignored = glob(ignore)?;
    let mut builder = ignore::gitignore::GitignoreBuilder::new(&cwd);
    if ignore_file.is_file()
        && let Some(error) = builder.add(ignore_file)
    {
        return Err(error.into());
    }
    let gitignored = builder.build()?;
    let defaults = vec![".".to_string()];
    let mut found = BTreeSet::new();
    for input in if paths.is_empty() { &defaults } else { paths } {
        let path = absolute(Path::new(input))?;
        if path.is_file() {
            if !ignored.is_match(&path)
                && !ignored.is_match(path.strip_prefix(&cwd).unwrap_or(&path))
                && !gitignored.matched_path_or_any_parents(&path, false).is_ignore()
                && !path.components().any(|component| {
                    ["node_modules", ".git", "dist", "target"]
                        .iter()
                        .any(|name| component.as_os_str() == *name)
                })
            {
                found.insert(path);
            }
            continue;
        }
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
                && !gitignored
                    .matched_path_or_any_parents(entry.path(), entry.file_type().is_dir())
                    .is_ignore()
        }) {
            let entry = entry?;
            if (entry.file_type().is_file()
                || (entry.file_type().is_symlink() && entry.path().is_file()))
                && matcher.is_match(entry.path())
            {
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
