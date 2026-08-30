//! Document generators for the pipeline fuzz lane.
//!
//! Kept beside the tests rather than inline so the lane itself stays
//! readable: this file is corpora and a mixer, nothing else.

/// xorshift64 — deterministic, so a failure reproduces from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Fragments that break scanners: openers with no closer, delimiters with
/// nothing to pair with, and the characters that make a byte index stop
/// being a character index.
const SOUP: &[&str] = &[
    "---\n",
    "title: x\n",
    "a: [1,2\n",
    "\n",
    "# ",
    "## ",
    "###### ",
    "\t",
    "  ",
    "```",
    "~~~",
    "js\n",
    "x\n",
    ":::",
    "tip",
    "warning",
    "details",
    "::: ",
    ":::steps",
    ":::code-group",
    "|",
    "-",
    ":",
    "*",
    "_",
    "~",
    "`",
    "[",
    "]",
    "(",
    ")",
    "{",
    "}",
    "<",
    ">",
    "!",
    "#",
    "$",
    "\\",
    "&",
    ";",
    "\"",
    "'",
    "/",
    "@",
    "^",
    "+",
    "=",
    "?",
    "%",
    "\r",
    "\u{0}",
    "\u{a0}",
    "\u{3042}",
    "\u{1F600}",
    "\u{0301}",
    "http://a.b/c",
    "https://x.y/z",
    "mailto:a@b",
    "[^1]",
    "[^1]:",
    "[[wiki]]",
    ":smile:",
    "$x$",
    "$$",
    "@[a]",
    "!!!",
    "<Foo",
    "</Foo>",
    "<Foo />",
    "{expr}",
    "import x from 'y'",
    "export const a = 1",
    "::: v-pre",
    "<!-- -->",
    "<!--@include: ../x.md-->",
    "^[inline note]",
    "{#id}",
    "{.cls}",
    "[[toc]]",
    "==mark==",
    "++ins++",
    "term\n: def",
    "- [ ] ",
    "> ",
    "1. ",
    "  - ",
    "<kbd>",
    "[a](b)",
    "![a](b)",
    "```mermaid\n",
    "```play\n",
    "<script>",
    "</script>",
    "\u{202E}",
    "\u{FEFF}",
    "text",
    "*[abbr]: x",
    "npm:pkg",
    "gh:a/b",
    "@user",
    "#42",
    "a.md#f",
    "<div",
    "</div>",
    "<td>",
    "| --- |",
    "|:--|--:|",
    "= ",
    "== ",
    "%%",
    "[!code focus]",
    "// [!code ++]",
    "{1,3-5}",
    "title=\"a\"",
    "[a]",
    "[a][b]",
    "![](",
    "()",
    "[]()",
    "<a href=",
    "\u{2028}",
    "\u{2029}",
];

/// Block templates, each with `{t}` holes filled from [`INLINES`].
const BLOCKS: &[&str] = &[
    "# {t}\n\n",
    "## {t} {#id}\n\n",
    "{t}\n\n",
    "- {t}\n- {t}\n\n",
    "1. {t}\n2. {t}\n\n",
    "> {t}\n> {t}\n\n",
    "```ts\n{t}\n```\n\n",
    "```ts title=\"a.ts\" {1,3-4}\n{t}\n```\n\n",
    "| {t} | {t} |\n| --- | ---: |\n| {t} | {t} |\n\n",
    "::: tip {t}\n{t}\n:::\n\n",
    "::: details\n{t}\n:::\n\n",
    "::: code-group\n```ts [a]\n{t}\n```\n```js [b]\n{t}\n```\n:::\n\n",
    "::: steps\n1. {t}\n2. {t}\n:::\n\n",
    "::: timeline\n- 2020-01-01 {t}\n:::\n\n",
    "::: file-tree\n- src\n  - {t}\n:::\n\n",
    "::: cards\n- {t}\n:::\n\n",
    "::: gallery\n![{t}](a.png)\n:::\n\n",
    "{t}\n: {t}\n: {t}\n\n",
    "*[{t}]: {t}\n\n",
    "[^{t}]: {t}\n\n",
    "[{t}]: /u \"{t}\"\n\n",
    "---\ntitle: {t}\n---\n\n",
    "<div>{t}</div>\n\n",
    "<Comp prop=\"{t}\">{t}</Comp>\n\n",
    "{ {t} }\n\n",
    "import x from '{t}'\n\n",
    "export const a = {t}\n\n",
    "$$\n{t}\n$$\n\n",
    "@[youtube]({t})\n\n",
    "!!! note\n{t}\n!!!\n\n",
    "[[toc]]\n\n",
    "<!-- {t} -->\n\n",
    "    {t}\n\n",
    "***\n\n",
    "{t}\n===\n\n",
];

const INLINES: &[&str] = &[
    "text",
    "*em*",
    "**strong**",
    "`code`",
    "[l](u)",
    "![i](s)",
    "~~del~~",
    "[[wiki]]",
    ":smile:",
    "$x^2$",
    "[^1]",
    "^[note]",
    "==mark==",
    "<kbd>K</kbd>",
    "{#id}",
    "{.cls}",
    "https://a.b/c",
    "OX-12",
    "a@b.co",
    "\u{3042}\u{3044}",
    "\u{1F600}",
    "\u{FEFF}",
    "\u{202E}",
    "\u{0}",
    "\u{0301}",
    "<Comp />",
    "{expr}",
    "\\*",
    "&amp;",
    "&#65;",
    "&notreal;",
    "|",
    "<",
    ">",
    "\"",
    "'",
    "npm:react",
    "gh:a/b",
    "@user",
    "#123",
    "a.md",
    "./x.md#f",
    "\t",
    "  ",
    "---",
    "...",
];

/// Even seeds build token soup, odd seeds build structured documents.
pub fn document(seed: u64) -> String {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    if seed.is_multiple_of(2) {
        let pieces = 1 + rng.below(26);
        let mut source = String::new();
        for _ in 0..pieces {
            source.push_str(SOUP[rng.below(SOUP.len())]);
        }
        return source;
    }
    let blocks = 1 + rng.below(6);
    let mut source = String::new();
    for _ in 0..blocks {
        let mut block = BLOCKS[rng.below(BLOCKS.len())].to_string();
        while let Some(at) = block.find("{t}") {
            block.replace_range(at..at + 3, INLINES[rng.below(INLINES.len())]);
        }
        source.push_str(&block);
    }
    source
}
