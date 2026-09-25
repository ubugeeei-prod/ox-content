/// Byte-for-byte copy of `sampleMarkdown` in `parse-benchmark-bun.mjs`,
/// including the leading and trailing newline. The JS harness derives
/// throughput from `input.length` (UTF-16 code units), which equals the byte
/// length here because the sample is pure ASCII.
pub(super) const SAMPLE_MARKDOWN: &str = r#"
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
  - Nested item
- List item 3

### Code Block

```javascript
function hello() {
  console.log("Hello, World!");
}
```

> This is a blockquote
> with multiple lines

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |

Here's a [link](https://example.com) and an image: ![alt](image.png)

---

Final paragraph with `inline code` and more text.
"#;
