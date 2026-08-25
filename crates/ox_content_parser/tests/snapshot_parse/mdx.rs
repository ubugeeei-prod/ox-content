use ox_content_parser::ParserOptions;

use super::check;

#[test]
fn snapshot_mdx_document_with_esm_components_expressions_and_fence() {
    check(
        "mdx_document_with_esm_components_expressions_and_fence",
        r#"import { Callout, Badge } from './ui'
export const meta = { title: 'MDX' }

# MDX Surface

Hello <Badge tone="info">{status.label}</Badge> world.

<Callout title="Install" count={steps.length} {...calloutProps}>

1. Install the CLI
2. Run **build**

```sh
npm i -g ox-content
```

</Callout>
"#,
        ParserOptions::mdx(),
    );
}

#[test]
fn snapshot_mdx_nested_member_components_and_fragments() {
    check(
        "mdx_nested_member_components_and_fragments",
        r#"<Docs.Layout>
<Docs.Header eyebrow="Guide">
<>Build <Icons.Sparkle /> faster</>
</Docs.Header>

<Docs.Body>
Use <Package.Name scope="@ox-content" /> with {runtime}.
</Docs.Body>
</Docs.Layout>
"#,
        ParserOptions::mdx(),
    );
}

#[test]
fn snapshot_mdx_unclosed_constructs_remain_markdown_recoverable() {
    check(
        "mdx_unclosed_constructs_remain_markdown_recoverable",
        r#"<Callout>

# The heading survives

Text before {broken

<Card title="unterminated"

```mdx
<RealExample />
```
"#,
        ParserOptions::mdx(),
    );
}

#[test]
fn snapshot_mdx_disabled_keeps_component_like_source_literal() {
    check(
        "mdx_disabled_keeps_component_like_source_literal",
        r#"<Callout title="Install">

Hello <Badge /> and {value}.

</Callout>
"#,
        ParserOptions::default(),
    );
}
