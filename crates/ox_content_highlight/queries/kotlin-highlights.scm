; Highlight query for tree-sitter-kotlin-ng 1.1.0.
; The published crate ships LANGUAGE only; these node names match that grammar
; (identifier / number_literal / line_comment), not the later Helix set.

[
  (line_comment)
  (block_comment)
  (shebang)
] @comment

[
  (string_literal)
  (multiline_string_literal)
  (character_literal)
] @string

(string_content) @string
(escape_sequence) @string.escape
(interpolation) @punctuation.special

[
  (number_literal)
  (float_literal)
] @number

[
  "fun"
  "val"
  "var"
  "class"
  "object"
  "interface"
  "companion"
  "enum"
  "typealias"
  "package"
  "import"
  "if"
  "else"
  "when"
  "for"
  "do"
  "while"
  "try"
  "catch"
  "finally"
  "throw"
  "return"
  "return@"
  "init"
  "constructor"
  "get"
  "set"
  "by"
  "where"
  "as"
  "as?"
  "in"
  "!in"
  "is"
  "!is"
] @keyword

[
  (class_modifier)
  (member_modifier)
  (function_modifier)
  (property_modifier)
  (platform_modifier)
  (variance_modifier)
  (parameter_modifier)
  (visibility_modifier)
  (reification_modifier)
  (inheritance_modifier)
] @keyword

(annotation) @attribute
(file_annotation) @attribute
(label) @label

(function_declaration
  name: (identifier) @function)

(class_declaration
  name: (identifier) @type)

(object_declaration
  name: (identifier) @type)

(type_alias
  type: (identifier) @type)

(enum_entry
  (identifier) @constant)

(user_type
  (identifier) @type)

(call_expression
  .
  (identifier) @function)

(parameter
  (identifier) @variable.parameter)

(variable_declaration
  (identifier) @variable)

(this_expression) @variable.builtin
(super_expression) @variable.builtin

((identifier) @constant.builtin
  (#any-of? @constant.builtin "true" "false" "null"))

((identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]*$"))

(identifier) @variable

[
  "."
  ","
  ";"
  ":"
  "::"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  "!"
  "!!"
  "!="
  "!=="
  "="
  "=="
  "==="
  ">"
  ">="
  "<"
  "<="
  "||"
  "&&"
  "+"
  "++"
  "+="
  "-"
  "--"
  "-="
  "*"
  "*="
  "/"
  "/="
  "%"
  "%="
  "?."
  "?:"
  ".."
  "..<"
  "->"
] @operator
