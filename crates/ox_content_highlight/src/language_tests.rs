use super::*;
use crate::test_support::{assert_text_has_capture_token, visible_text};

#[test]
fn fish_highlights_functions_set_substitution_variables_and_pipelines() {
    let code = r#"function fish_prompt
  set -l branch (git branch --show-current)
  echo "$branch < &" | string upper
  # production corpus shape
end
"#;

    let html = highlight_to_html(code, "fish").expect("fish is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "function", "keyword");
    assert_text_has_capture_token(&html, "fish_prompt", "function");
    assert_text_has_capture_token(&html, "set", "function");
    assert_text_has_capture_token(&html, "git", "function");
    assert_text_has_capture_token(&html, "$branch", "constant");
    assert_text_has_capture_token(&html, "|", "operator");
    assert_text_has_capture_token(&html, " < &\"", "string");
    assert_text_has_capture_token(&html, "# production corpus shape", "comment");
    assert!(html.contains("&lt;"), "{html}");
    assert!(html.contains("&amp;"), "{html}");
}
