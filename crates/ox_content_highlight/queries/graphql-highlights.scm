; Source: Helix / nvim-treesitter GraphQL highlights, capture names
; aligned to this crate. tree-sitter-graphql 0.2.0 ships LANGUAGE only.

(scalar_type_definition (name) @type)
(object_type_definition (name) @type)
(interface_type_definition (name) @type)
(union_type_definition (name) @type)
(enum_type_definition (name) @type)
(input_object_type_definition (name) @type)
(directive_definition (name) @attribute)
(directive_definition "@" @attribute)
(scalar_type_extension (name) @type)
(object_type_extension (name) @type)
(interface_type_extension (name) @type)
(union_type_extension (name) @type)
(enum_type_extension (name) @type)
(input_object_type_extension (name) @type)
(named_type (name) @type)
(directive) @attribute

(field (name) @property)
(field (alias (name) @property))
(field_definition (name) @property)
(object_value (object_field (name) @property))
(enum_value (name) @constant)

(operation_definition (name) @variable)
(fragment_name (name) @variable)
(input_fields_definition (input_value_definition (name) @variable.parameter))
(argument (name) @variable.parameter)
(arguments_definition (input_value_definition (name) @variable.parameter))
(variable_definition (variable) @variable.parameter)
(argument (value (variable) @variable))

(string_value) @string
(int_value) @number
(float_value) @number
(boolean_value) @constant.builtin
(null_value) @constant.builtin
(description) @comment
(comment) @comment

[
  "query"
  "mutation"
  "subscription"
  "fragment"
  "scalar"
  "type"
  "interface"
  "union"
  "enum"
  "input"
  "extend"
  "directive"
  "schema"
  "on"
  "repeatable"
  "implements"
] @keyword

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

"=" @operator
"|" @punctuation.delimiter
"&" @punctuation.delimiter
":" @punctuation.delimiter
"..." @punctuation.special
"!" @punctuation.special
