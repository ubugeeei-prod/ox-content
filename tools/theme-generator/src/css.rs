use regex::Regex;
use std::sync::LazyLock;

static COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").expect("valid comment regex"));
static NEWLINE_WHITESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*\n\s*").expect("valid newline whitespace regex"));
static EXTRA_NEWLINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{2,}").expect("valid extra newline regex"));
static PUNCTUATION_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*([{};,>])\s*").expect("valid punctuation regex"));
static COLON_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w):\s*").expect("valid colon regex"));

pub(crate) fn minify(source: &str) -> String {
    let output = COMMENT.replace_all(source, "");
    let output = NEWLINE_WHITESPACE.replace_all(&output, "\n");
    let output = EXTRA_NEWLINES.replace_all(&output, "\n");
    let output = PUNCTUATION_SPACE.replace_all(&output, "$1");
    let output = COLON_SPACE.replace_all(&output, "$1:");
    output.replace(";}", "}").trim().to_string()
}

pub(crate) fn escape_template_literal(css: &str) -> String {
    css.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${")
}

#[cfg(test)]
mod tests {
    use super::{escape_template_literal, minify};

    #[test]
    fn minifies_without_rewriting_descendant_not_selectors() {
        assert_eq!(minify(".content :not(pre) { color: red; }\n"), ".content :not(pre){color:red}");
    }

    #[test]
    fn escapes_template_literal_interpolation() {
        assert_eq!(escape_template_literal("a{content:`${x}\\`}"), "a{content:\\`\\${x}\\\\\\`}");
    }
}
