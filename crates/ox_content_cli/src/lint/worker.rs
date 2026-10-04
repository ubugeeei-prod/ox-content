use crate::Result;
use ox_content_markdown_lint::{MarkdownLintResult, MarkdownLinter};
use std::{fs, io::Write, path::Path};

pub(super) fn check(
    path: &str,
    stdin: Option<&str>,
    linter: &MarkdownLinter,
    fix: bool,
) -> Result<(MarkdownLintResult, u32)> {
    let source = if let Some(input) = stdin {
        std::borrow::Cow::Borrowed(input)
    } else {
        std::borrow::Cow::Owned(fs::read_to_string(path)?)
    };
    if !fix {
        return Ok((linter.lint_without_mask(&source), 0));
    }
    let fixed = linter.fix(&source);
    if fixed.applied_fixes > 0 {
        write_fixed(Path::new(path), &source, &fixed.output)?;
    }
    Ok((fixed.result, fixed.applied_fixes))
}

fn write_fixed(path: &Path, original: &str, output: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(format!("Refusing to fix non-regular file: {}", path.display()).into());
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
    temporary.as_file().set_permissions(metadata.permissions())?;
    temporary.write_all(output.as_bytes())?;
    // Avoid overwriting edits made after the worker read the document.
    if fs::read_to_string(path)? != original {
        return Err(format!("File changed while linting: {}", path.display()).into());
    }
    temporary.persist(path)?;
    Ok(())
}
