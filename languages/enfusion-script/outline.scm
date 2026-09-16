(decl_class
  (class_modifier)* @context
  "class" @context
  typename: (identifier) @name) @item

(decl_enum
  "enum" @context
  typename: (identifier) @name) @item

(enum_member
  name: (identifier) @name) @item

(class_body
  (decl_field
    (field_modifier)* @context
    type: (_) @context
    name: (identifier) @name) @item)

(decl_method
  (method_modifier)* @context
  return_type: (_) @context
  name: (identifier) @name
  "(" @context
  (formal_parameters
    (formal_parameter
      type: (_) @context
      name: (identifier) @context))? @context
  ")" @context) @item

(typedef
  "typedef" @context
  alias: (identifier) @name) @item
