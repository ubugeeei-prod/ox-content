(identifier) @variable

((identifier) @constant
  (#match? @constant "^[A-Z][A-Z_0-9]*$"))

[
  "if"
  "else"
  "elseif"
  "endif"
  "try"
  "catch"
  "finally"
  "endtry"
  "throw"
  "for"
  "endfor"
  "in"
  "while"
  "endwhile"
  "break"
  "continue"
  "function"
  "endfunction"
] @keyword

(function_declaration
  name: (_) @function)

(function_declaration
  name:
    (scoped_identifier
      (identifier) @function))

(call_expression
  function: (identifier) @function)

(call_expression
  function:
    (scoped_identifier
      (identifier) @function))

(parameters
  (identifier) @variable.parameter)

(default_parameter
  (identifier) @variable.parameter)

[
  (bang)
  (spread)
] @punctuation.special

[
  (no_option)
  (inv_option)
  (default_option)
  (option_name)
] @constant.builtin

[
  (scope)
  "a:"
  "$"
] @module

[
  "let"
  "unlet"
  "const"
  "call"
  "execute"
  "normal"
  "set"
  "setfiletype"
  "setlocal"
  "silent"
  "echo"
  "echon"
  "echohl"
  "echomsg"
  "echoerr"
  "autocmd"
  "augroup"
  "return"
  "syntax"
  "filetype"
  "source"
  "lua"
  "ruby"
  "perl"
  "python"
  "highlight"
  "command"
  "delcommand"
  "comclear"
  "colorscheme"
  "scriptencoding"
  "startinsert"
  "stopinsert"
  "global"
  "runtime"
  "wincmd"
  "cnext"
  "cprevious"
  "cNext"
  "vertical"
  "leftabove"
  "aboveleft"
  "rightbelow"
  "belowright"
  "topleft"
  "botright"
  (unknown_command_name)
  "edit"
  "enew"
  "find"
  "ex"
  "visual"
  "view"
  "eval"
  "sign"
] @keyword

(map_statement
  cmd: _ @keyword)

(keycode) @constant.builtin

(command_name) @function.builtin

(filetype_statement
  [
    "detect"
    "plugin"
    "indent"
    "on"
    "off"
  ] @keyword)

(syntax_statement
  (keyword) @string)

(syntax_statement
  [
    "enable"
    "on"
    "off"
    "reset"
    "case"
    "spell"
    "foldlevel"
    "iskeyword"
    "keyword"
    "match"
    "cluster"
    "region"
    "clear"
    "include"
  ] @keyword)

(syntax_argument
  name: _ @keyword)

[
  "<buffer>"
  "<nowait>"
  "<silent>"
  "<script>"
  "<expr>"
  "<unique>"
] @constant.builtin

(augroup_name) @module

(au_event) @constant

(normal_statement
  (commands) @constant)

(hl_attribute
  key: _ @property
  val: _ @constant)

(hl_group) @type

(highlight_statement
  [
    "default"
    "link"
    "clear"
  ] @keyword)

(command) @string

(command_attribute
  name: _ @property)

(command_attribute
  val: (behavior
         _ @constant))

(plus_plus_opt
  val: _? @constant) @property

(plus_cmd
  "+" @property) @property

(runtime_statement
  (where) @operator)

(colorscheme_statement
  (name) @string)

(scriptencoding_statement
  (encoding) @string.special)

(string_literal) @string

(integer_literal) @number

(float_literal) @number

(comment) @comment

(line_continuation_comment) @comment

(pattern) @string.special

(pattern_multi) @string.special

(filename) @string.special

(heredoc
  (body) @string)

(heredoc
  (parameter) @keyword)

[
  (marker_definition)
  (endmarker)
] @label
