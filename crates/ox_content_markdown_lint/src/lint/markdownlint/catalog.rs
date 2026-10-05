// Rule metadata follows markdownlint 0.41.1 (MIT); implementations are native Rust.
#[derive(serde::Serialize)]
pub struct MarkdownlintRule {
    pub number: usize,
    pub id: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    pub tags: &'static [&'static str],
}
pub(super) const RULES: &[MarkdownlintRule] = &[
    rule(
        1,
        "MD001",
        "Heading levels should only increment by one level at a time",
        &["heading-increment"],
        &["headings"],
    ),
    rule(3, "MD003", "Heading style", &["heading-style"], &["headings"]),
    rule(4, "MD004", "Unordered list style", &["ul-style"], &["bullet", "ul"]),
    rule(
        5,
        "MD005",
        "Inconsistent indentation for list items at the same level",
        &["list-indent"],
        &["bullet", "ul", "indentation"],
    ),
    rule(
        7,
        "MD007",
        "Unordered list indentation",
        &["ul-indent"],
        &["bullet", "ul", "indentation"],
    ),
    rule(9, "MD009", "Trailing spaces", &["no-trailing-spaces"], &["whitespace"]),
    rule(10, "MD010", "Hard tabs", &["no-hard-tabs"], &["whitespace", "hard_tab"]),
    rule(11, "MD011", "Reversed link syntax", &["no-reversed-links"], &["links"]),
    rule(
        12,
        "MD012",
        "Multiple consecutive blank lines",
        &["no-multiple-blanks"],
        &["whitespace", "blank_lines"],
    ),
    rule(13, "MD013", "Line length", &["line-length"], &["line_length"]),
    rule(
        14,
        "MD014",
        "Dollar signs used before commands without showing output",
        &["commands-show-output"],
        &["code"],
    ),
    rule(
        18,
        "MD018",
        "No space after hash on atx style heading",
        &["no-missing-space-atx"],
        &["headings", "atx", "spaces"],
    ),
    rule(
        19,
        "MD019",
        "Multiple spaces after hash on atx style heading",
        &["no-multiple-space-atx"],
        &["headings", "atx", "spaces"],
    ),
    rule(
        20,
        "MD020",
        "No space inside hashes on closed atx style heading",
        &["no-missing-space-closed-atx"],
        &["headings", "atx_closed", "spaces"],
    ),
    rule(
        21,
        "MD021",
        "Multiple spaces inside hashes on closed atx style heading",
        &["no-multiple-space-closed-atx"],
        &["headings", "atx", "spaces"],
    ),
    rule(
        22,
        "MD022",
        "Headings should be surrounded by blank lines",
        &["blanks-around-headings"],
        &["headings", "blank_lines"],
    ),
    rule(
        23,
        "MD023",
        "Headings must start at the beginning of the line",
        &["heading-start-left"],
        &["headings", "spaces"],
    ),
    rule(
        24,
        "MD024",
        "Multiple headings with the same content",
        &["no-duplicate-heading"],
        &["headings"],
    ),
    rule(
        25,
        "MD025",
        "Multiple top-level headings in the same document",
        &["single-title", "single-h1"],
        &["headings"],
    ),
    rule(
        26,
        "MD026",
        "Trailing punctuation in heading",
        &["no-trailing-punctuation"],
        &["headings"],
    ),
    rule(
        27,
        "MD027",
        "Multiple spaces after blockquote symbol",
        &["no-multiple-space-blockquote"],
        &["blockquote", "whitespace", "indentation"],
    ),
    rule(
        28,
        "MD028",
        "Blank line inside blockquote",
        &["no-blanks-blockquote"],
        &["blockquote", "whitespace"],
    ),
    rule(29, "MD029", "Ordered list item prefix", &["ol-prefix"], &["ol"]),
    rule(
        30,
        "MD030",
        "Spaces after list markers",
        &["list-marker-space"],
        &["ol", "ul", "whitespace"],
    ),
    rule(
        31,
        "MD031",
        "Fenced code blocks should be surrounded by blank lines",
        &["blanks-around-fences"],
        &["code", "blank_lines"],
    ),
    rule(
        32,
        "MD032",
        "Lists should be surrounded by blank lines",
        &["blanks-around-lists"],
        &["bullet", "ul", "ol", "blank_lines"],
    ),
    rule(33, "MD033", "Inline HTML", &["no-inline-html"], &["html"]),
    rule(34, "MD034", "Bare URL used", &["no-bare-urls"], &["links", "url"]),
    rule(35, "MD035", "Horizontal rule style", &["hr-style"], &["hr"]),
    rule(
        36,
        "MD036",
        "Emphasis used instead of a heading",
        &["no-emphasis-as-heading"],
        &["headings", "emphasis"],
    ),
    rule(
        37,
        "MD037",
        "Spaces inside emphasis markers",
        &["no-space-in-emphasis"],
        &["whitespace", "emphasis"],
    ),
    rule(
        38,
        "MD038",
        "Spaces inside code span elements",
        &["no-space-in-code"],
        &["whitespace", "code"],
    ),
    rule(39, "MD039", "Spaces inside link text", &["no-space-in-links"], &["whitespace", "links"]),
    rule(
        40,
        "MD040",
        "Fenced code blocks should have a language specified",
        &["fenced-code-language"],
        &["code", "language"],
    ),
    rule(
        41,
        "MD041",
        "First line in a file should be a top-level heading",
        &["first-line-heading", "first-line-h1"],
        &["headings"],
    ),
    rule(42, "MD042", "No empty links", &["no-empty-links"], &["links"]),
    rule(43, "MD043", "Required heading structure", &["required-headings"], &["headings"]),
    rule(
        44,
        "MD044",
        "Proper names should have the correct capitalization",
        &["proper-names"],
        &["spelling"],
    ),
    rule(
        45,
        "MD045",
        "Images should have alternate text (alt text)",
        &["no-alt-text"],
        &["accessibility", "images"],
    ),
    rule(46, "MD046", "Code block style", &["code-block-style"], &["code"]),
    rule(
        47,
        "MD047",
        "Files should end with a single newline character",
        &["single-trailing-newline"],
        &["blank_lines"],
    ),
    rule(48, "MD048", "Code fence style", &["code-fence-style"], &["code"]),
    rule(49, "MD049", "Emphasis style", &["emphasis-style"], &["emphasis"]),
    rule(50, "MD050", "Strong style", &["strong-style"], &["emphasis"]),
    rule(51, "MD051", "Link fragments should be valid", &["link-fragments"], &["links"]),
    rule(
        52,
        "MD052",
        "Reference links and images should use a label that is defined",
        &["reference-links-images"],
        &["images", "links"],
    ),
    rule(
        53,
        "MD053",
        "Link and image reference definitions should be needed",
        &["link-image-reference-definitions"],
        &["images", "links"],
    ),
    rule(54, "MD054", "Link and image style", &["link-image-style"], &["images", "links"]),
    rule(55, "MD055", "Table pipe style", &["table-pipe-style"], &["table"]),
    rule(56, "MD056", "Table column count", &["table-column-count"], &["table"]),
    rule(
        58,
        "MD058",
        "Tables should be surrounded by blank lines",
        &["blanks-around-tables"],
        &["table"],
    ),
    rule(
        59,
        "MD059",
        "Link text should be descriptive",
        &["descriptive-link-text"],
        &["accessibility", "links"],
    ),
    rule(60, "MD060", "Table column style", &["table-column-style"], &["table"]),
];
pub(super) fn mask(name: &str) -> u64 {
    RULES
        .iter()
        .filter(|rule| {
            rule.id.eq_ignore_ascii_case(name)
                || rule
                    .aliases
                    .iter()
                    .chain(rule.tags.iter())
                    .any(|alias| alias.eq_ignore_ascii_case(name))
        })
        .fold(0, |bits, rule| bits | (1 << rule.number))
}
pub(super) fn all() -> u64 {
    RULES.iter().fold(0, |bits, rule| bits | (1 << rule.number))
}

const fn rule(
    number: usize,
    id: &'static str,
    description: &'static str,
    aliases: &'static [&'static str],
    tags: &'static [&'static str],
) -> MarkdownlintRule {
    MarkdownlintRule { number, id, description, aliases, tags }
}
