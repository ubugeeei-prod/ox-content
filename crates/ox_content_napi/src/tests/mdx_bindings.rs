#[test]
fn mdx_flag_reaches_parse_and_transform_bindings() {
    let parsed = crate::parse(
        "import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n".to_string(),
        Some(crate::JsParserOptions { mdx: Some(true), ..Default::default() }),
    );
    let ast: serde_json::Value = serde_json::from_str(&parsed.ast).unwrap();
    assert_eq!(ast["children"][0]["type"], "mdxjsEsm");
    assert_eq!(ast["children"][1]["type"], "mdxJsxFlowElement");

    let transformed = crate::transform(
        "import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n".to_string(),
        Some(crate::JsTransformOptions { mdx: Some(true), ..Default::default() }),
    );
    assert!(transformed.errors.is_empty());
    assert!(!transformed.html.contains("import Alert"));
    assert!(transformed.html.contains("data-ox-island=\"Alert\""));
    assert_eq!(transformed.imports.len(), 1);
    assert_eq!(transformed.imports[0].source, "./Alert");
    assert_eq!(transformed.imports[0].specifiers.len(), 1);
    assert_eq!(transformed.imports[0].specifiers[0].imported, "default");
    assert_eq!(transformed.imports[0].specifiers[0].local, "Alert");
    assert_eq!(transformed.imports[0].specifiers[0].kind, "default");
    assert!(transformed.exports.is_empty());
    assert_eq!(transformed.components, vec!["Alert"]);
}

#[test]
fn mdx_transform_exports_module_metadata() {
    let transformed = crate::transform(
        concat!(
            "import Alert from './Alert'\n",
            "import { Chart as Plot } from './Chart'\n",
            "import * as Icons from './icons'\n",
            "export const title = 'Guide'\n",
            "export function helper() {}\n\n",
            "<Alert />\n",
            "Hello <Badge /> and <Icons.Star />\n",
        )
        .to_string(),
        Some(crate::JsTransformOptions { mdx: Some(true), ..Default::default() }),
    );

    assert!(transformed.errors.is_empty());
    assert_eq!(transformed.imports.len(), 3);
    assert_eq!(transformed.imports[0].source, "./Alert");
    assert_eq!(transformed.imports[1].source, "./Chart");
    assert_eq!(transformed.imports[1].specifiers[0].imported, "Chart");
    assert_eq!(transformed.imports[1].specifiers[0].local, "Plot");
    assert_eq!(transformed.imports[1].specifiers[0].kind, "named");
    assert_eq!(transformed.imports[2].specifiers[0].kind, "namespace");
    assert_eq!(transformed.exports, vec!["title".to_string(), "helper".to_string()]);
    assert_eq!(transformed.components, vec!["Alert", "Badge", "Icons.Star"]);
}

#[test]
fn mdx_metadata_is_empty_when_mdx_is_off() {
    let transformed = crate::transform(
        "import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n".to_string(),
        Some(crate::JsTransformOptions { mdx: Some(false), ..Default::default() }),
    );
    assert!(transformed.imports.is_empty());
    assert!(transformed.exports.is_empty());
    assert!(transformed.components.is_empty());
}
