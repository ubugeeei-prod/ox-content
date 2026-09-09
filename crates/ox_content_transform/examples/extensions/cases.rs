use ox_content_transform::*;
#[path = "blocks.rs"]
mod blocks;
#[path = "files.rs"]
mod files;
#[path = "syntax.rs"]
mod syntax;
pub const PROSE: &str = "# Ordinary documentation\n\nPlain prose and **emphasis**, a [link](https://example.com), and `code`.\n\n日本語の文章です。\n";
pub struct Case {
    pub name: &'static str,
    pub source: String,
    pub options: TransformOptions,
    pub marker: &'static str,
}
pub fn case(
    name: &'static str,
    source: &str,
    marker: &'static str,
    options: TransformOptions,
) -> Case {
    Case { name, source: source.repeat(32), marker, options }
}
pub fn cases() -> Vec<Case> {
    let mut cases = syntax::cases();
    cases.extend(blocks::cases());
    cases.extend(files::cases());
    cases
}
