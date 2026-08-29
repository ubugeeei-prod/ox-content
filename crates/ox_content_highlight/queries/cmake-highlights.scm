; Source: tree-sitter-cmake 0.7.4 queries/highlights.scm, reduced and
; normalized to Ox Content's supported capture names.

[
  (quoted_argument)
  (bracket_argument)
] @string

(escape_sequence) @string.escape

[
  (bracket_comment)
  (line_comment)
] @comment

(variable_ref) @variable.parameter
(variable) @variable.parameter

(normal_command
  (identifier) @function)

(normal_command
  (identifier) @function.builtin
  (#match? @function.builtin "^(cmake_minimum_required|project|add_executable|add_library|target_compile_definitions|target_include_directories|target_link_libraries|set|unset|list|message|find_package|include)$"))

[
  "ENV"
  "CACHE"
] @module

[
  "$"
  "{"
  "}"
] @punctuation.special

[
  "("
  ")"
] @punctuation.bracket

[
  (function)
  (endfunction)
  (macro)
  (endmacro)
  (if)
  (elseif)
  (else)
  (endif)
  (foreach)
  (endforeach)
  (while)
  (endwhile)
] @keyword

(normal_command
  (identifier) @keyword
  (#match? @keyword "^(continue|break|return)$"))

((argument) @constant.builtin
  (#match? @constant.builtin "^(1|ON|YES|TRUE|Y|0|OFF|NO|FALSE|N|IGNORE|NOTFOUND|.*-NOTFOUND)$"))

(if_command
  (if)
  (argument_list
    (argument) @operator)
  (#any-of? @operator
    "NOT" "AND" "OR" "COMMAND" "POLICY" "TARGET" "TEST" "DEFINED" "IN_LIST" "EXISTS" "IS_NEWER_THAN"
    "IS_DIRECTORY" "IS_SYMLINK" "IS_ABSOLUTE" "MATCHES" "LESS" "GREATER" "EQUAL" "LESS_EQUAL"
    "GREATER_EQUAL" "STRLESS" "STRGREATER" "STREQUAL" "STRLESS_EQUAL" "STRGREATER_EQUAL"
    "VERSION_LESS" "VERSION_GREATER" "VERSION_EQUAL" "VERSION_LESS_EQUAL" "VERSION_GREATER_EQUAL"))

(function_command
  (function)
  (argument_list
    .
    (argument) @function
    (argument)* @variable.parameter))

(macro_command
  (macro)
  (argument_list
    .
    (argument) @function
    (argument)* @variable.parameter))
