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

#[test]
fn cmake_highlights_commands_variables_generator_expressions_and_comments() {
    let code = r#"cmake_minimum_required(VERSION 3.28)
project(App)
set(SOURCES "main < &.cpp")
add_executable(app ${SOURCES})
target_compile_definitions(app PRIVATE "$<$<CONFIG:Debug>:DEBUG_BUILD>")
if(DEFINED SOURCES)
  message(STATUS "configured")
endif()
# production corpus shape
"#;

    let html = highlight_to_html(code, "cmake").expect("cmake is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "cmake_minimum_required", "function.builtin");
    assert_text_has_capture_token(&html, "project", "function.builtin");
    assert_text_has_capture_token(&html, "set", "function.builtin");
    assert_text_has_capture_token(&html, "add_executable", "function.builtin");
    assert_text_has_capture_token(&html, "SOURCES", "variable.parameter");
    assert_text_has_capture_token(&html, "\"main < &.cpp\"", "string");
    assert_text_has_capture_token(&html, "$", "punctuation.special");
    assert_text_has_capture_token(&html, "<CONFIG:Debug>:DEBUG_BUILD>\"", "string");
    assert_text_has_capture_token(&html, "if", "keyword");
    assert_text_has_capture_token(&html, "DEFINED", "operator");
    assert_text_has_capture_token(&html, "endif", "keyword");
    assert_text_has_capture_token(&html, "# production corpus shape", "comment");
    assert!(html.contains("&lt;"), "{html}");
    assert!(html.contains("&amp;"), "{html}");
}

#[test]
fn vimscript_highlights_functions_commands_ranges_strings_and_comments() {
    let code = r#"function! s:Run(cmd) abort
  let l:output = execute(a:cmd)
  1,3s/foo/bar/g
  lua print("hi")
  echo "done < &"
  " production corpus shape
endfunction
"#;

    let html = highlight_to_html(code, "vimscript").expect("vimscript is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "function", "keyword");
    assert_text_has_capture_token(&html, "Run", "function");
    assert_text_has_capture_token(&html, "cmd", "variable.parameter");
    assert_text_has_capture_token(&html, "let", "keyword");
    assert_text_has_capture_token(&html, "l:", "module");
    assert_text_has_capture_token(&html, "execute", "function");
    assert_text_has_capture_token(&html, "1", "number");
    assert_text_has_capture_token(&html, "foo/bar/g", "keyword");
    assert_text_has_capture_token(&html, "lua", "keyword");
    assert_text_has_capture_token(&html, "echo", "keyword");
    assert_text_has_capture_token(&html, "\"done < &\"", "string");
    assert_text_has_capture_token(&html, "\" production corpus shape", "comment");
    assert_text_has_capture_token(&html, "endfunction", "keyword");
    assert!(html.contains("&lt;"), "{html}");
    assert!(html.contains("&amp;"), "{html}");
}
