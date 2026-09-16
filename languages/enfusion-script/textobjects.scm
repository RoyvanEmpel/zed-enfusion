(decl_method
  body: (block "{" (_)* @function.inside "}")) @function.around

(decl_method) @function.around

(decl_class
  body: (class_body "{" (_)* @class.inside "}")) @class.around

(decl_enum
  body: (enum_body "{" (_)* @class.inside "}")) @class.around

[
  (comment_line)
  (comment_block)
  (doc_line)
  (doc_block)
]+ @comment.around
