; Enfusion Script (Enforce) highlighting for Zed.
; Later patterns take precedence over earlier ones, so the generic
; (identifier) capture comes first and specific captures follow.

(identifier) @variable

; Comments
[
  (comment_line)
  (comment_block)
] @comment

[
  (doc_line)
  (doc_block)
] @comment.doc

; Literals
(literal_bool) @boolean
(literal_int) @number
(literal_float) @number
(literal_string) @string
(escape_sequence) @string.escape
(literal_null) @constant.builtin

; Preprocessor
[
  "#include"
  "#define"
  "#ifdef"
  "#ifndef"
  (preproc_else)
  (preproc_endif)
] @preproc

(preproc_const) @constant

; Operators
[
  "+" "-" "*" "/" "%"
  "++" "--"
  "=" "+=" "-=" "*=" "/=" "&=" "^=" "|=" "<<=" ">>="
  "<" "<=" ">=" ">" "==" "!="
  "!" "&&" "||"
  ">>" "<<" "&" "|" "^" "~"
] @operator

; Punctuation
[
  "(" ")"
  "[" "]"
  "{" "}"
] @punctuation.bracket

(type_parameters ["<" ">"] @punctuation.bracket)
(types ["<" ">"] @punctuation.bracket)

[
  ","
  "."
  ":"
  ";"
] @punctuation.delimiter

; Keywords
[
  "class"
  "enum"
  "typedef"
  "extends"
  "new"
  "delete"
  "return"
  "if"
  "else"
  "switch"
  "case"
  "default"
  "while"
  "for"
  "foreach"
  "continue"
  "break"
  "ref"
  "auto"
] @keyword

[
  (variable_modifier)
  (method_modifier)
  (class_modifier)
  (field_modifier)
  (formal_parameter_modifier)
] @keyword

; Types
(type_primitive) @type.builtin

(type_identifier (identifier) @type)

(type_parameter name: (identifier) @type)

(decl_class typename: (identifier) @type)

(decl_class
  superclass: (superclass typename: (identifier) @type))

(decl_enum typename: (identifier) @enum)

(decl_enum superenum: (identifier) @enum)

(typedef alias: (identifier) @type)

[
  (super)
  (this)
] @variable.special

; Fields and members
(decl_field name: (identifier) @property)

(member_access member: (identifier) @property)

; Constants: SCREAMING_CASE identifiers, const fields and enum members
((identifier) @constant
  (#match? @constant "^[A-Z_][A-Z0-9_]+$"))

(decl_field
  (field_modifier) @_modifier
  name: (identifier) @constant
  (#eq? @_modifier "const"))

(enum_member name: (identifier) @constant)

; Parameters
(formal_parameter name: (identifier) @variable.parameter)

(actual_parameter name: (identifier) @variable.parameter)

; Functions and methods
(decl_method name: (identifier) @function.method)

(invokation invoked: (identifier) @function)

; Constructor and destructor share the class name
(decl_class
  typename: (identifier) @_classname
  body: (class_body
    (decl_method
      name: (identifier) @constructor
      (#eq? @constructor @_classname))))

; Attributes: [Attribute(...)], [RplProp(...)], [BaseContainerProps()]
(attribute_list
  (invokation invoked: (identifier) @attribute))

(attribute_list (identifier) @attribute)

(attribute_list
  (type_identifier (identifier) @attribute))
