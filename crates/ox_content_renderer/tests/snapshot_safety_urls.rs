#[path = "support/snapshot.rs"]
mod snapshot_support;

use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
use snapshot_support::check;

#[test]
fn html_sanitize_escapes_html_block() {
    check(
        "sanitize_escapes_html_block",
        "<div><script>alert(1)</script></div>\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_escapes_inline_raw_html() {
    check(
        "sanitize_escapes_inline_raw_html",
        "<span>ok</span>\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_neutralizes_javascript_link() {
    check(
        "sanitize_neutralizes_javascript_link",
        "[run](javascript:alert(1))\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_neutralizes_obfuscated_javascript_link() {
    check(
        "sanitize_neutralizes_obfuscated_javascript_link",
        "[run](  JaVa ScRiPt:alert(1))\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_clears_unsafe_image_data_url() {
    check(
        "sanitize_clears_unsafe_image_data_url",
        "![x](data:text/html,<script>alert(1)</script>)\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_keeps_allowed_schemes() {
    check(
        "sanitize_keeps_allowed_schemes",
        "[guide](./guide.md) [mail](mailto:hi@example.com) [phone](tel:+123)\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

// --- Base URL / md-link conversion ---

#[test]
fn html_base_url_prefixes_root_absolute_links() {
    check(
        "base_url_prefixes_root_absolute_links",
        "[guide](/guide) [dir](/guide/) [md](/api.md#types)\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_prefixes_root_absolute_images() {
    check(
        "base_url_prefixes_root_absolute_images",
        "![logo](/img/logo.png)\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_prefixes_raw_html_attrs() {
    check(
        "base_url_prefixes_raw_html_attrs",
        "<div>\n<a href=\"/guide\">Guide</a>\n<img src='/img/logo.png'>\n</div>\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_leaves_protocol_relative_unchanged() {
    check(
        "base_url_leaves_protocol_relative_unchanged",
        "<script src=\"//cdn.example/app.js\"></script>\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

// --- Tables (GFM) ---
