use super::check;

#[test]
fn accepts_balanced_components_and_quoted_props() {
    let diagnostics = check("<Alert tone=\"info\" :count={count}>Hello</Alert>\n<Card />");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn reports_mismatched_and_unquoted_component_props() {
    let diagnostics = check("<Alert tone=info><Panel></Alert>");
    let messages: Vec<&str> =
        diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect();

    assert!(messages.iter().any(|message| message.contains("quoted strings")));
    assert!(messages.iter().any(|message| message.contains("does not match")));
    assert!(messages.iter().any(|message| message.contains("missing a closing tag")));
}

#[test]
fn ignores_component_like_text_inside_fences() {
    let diagnostics = check("```mdc\n<Alert tone=bad>\n```\n<Alert />");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
