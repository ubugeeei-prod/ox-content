; Highlight query for tree-sitter-hcl 1.1.0.
; The published crate ships LANGUAGE with no queries, so these patterns are
; written against its own node-types.json. Two shapes drive the file: every
; literal string is a `string_lit` wrapping a `template_literal`, and block
; headers, attribute names, and function names are all bare `identifier`
; nodes told apart only by their parent.

(comment) @comment

(numeric_lit) @number

[
  (bool_lit)
  (null_lit)
] @constant.builtin

[
  (string_lit)
  (heredoc_template)
  (template_literal)
] @string

; Capture only the `${` / `}` delimiters. Capturing the whole
; `template_interpolation` would repaint the expression inside it as
; punctuation and lose every token the interpolation actually contains.
[
  (template_interpolation_start)
  (template_interpolation_end)
  (template_directive_start)
  (template_directive_end)
] @punctuation.special

(block (identifier) @keyword)
(attribute (identifier) @property)
(function_call (identifier) @function)
(get_attr (identifier) @property)
(variable_expr (identifier) @variable)

[
  "for"
  "in"
  "if"
  "else"
  "endif"
  "endfor"
] @keyword

[
  "{"
  "}"
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  "."
  ","
  ":"
] @punctuation.delimiter

[
  "="
  "=>"
  "?"
  "!"
  "!="
  "=="
  "<"
  "<="
  ">"
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
  "&&"
  "||"
] @operator
