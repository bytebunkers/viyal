import re

with open('compiler/codegen/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Replace Expr::New
content = re.sub(
    r'Expr::New\(class_name, _, args\) => \{\s*// MVP:.*?self\.output\.push_str\(&format\!\("\(\(struct \{\}\*\)calloc\(1, sizeof\(struct \{\}\)\)\)", class_name, class_name\)\);\s*// Note:.*?\n\s*\}',
    r'''Expr::New(class_name, _, args) => {
                self.output.push_str(&format!("{}_new(", class_name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.visit_expr(&arg.node)?;
                }
                self.output.push_str(")");
            }''',
    content,
    flags=re.DOTALL
)

# 2. Replace Expr::Call
call_regex = r'''Expr::Call\(callee, _, args\) => \{\s*if let Expr::Identifier\(id\) = &callee\.node \{\s*if id == "print" \{\s*self\.output\.push_str\("printf\("\);\s*if let Some\(arg\) = args\.first\(\) \{\s*if let Expr::Literal\(Literal::String\(s\)\) = &arg\.node \{\s*write!\(&mut self\.output, "\\"\{\}\\\\n\\"", s\)\.unwrap\(\);\s*\} else \{\s*self\.output\.push_str\("\"%ld\\\\n\", "\);\s*self\.visit_expr\(&arg\.node\)\?;\s*\}\s*\}\s*self\.output\.push_str\("\)"\);\s*return Ok\(\(\)\);\s*\}\s*self\.output\.push_str\(id\);\s*\} else \{\s*self\.visit_expr\(&callee\.node\)\?;\s*\}\s*self\.output\.push_str\("\("\);\s*for \(i, arg\) in args\.iter\(\)\.enumerate\(\) \{\s*if i > 0 \{\s*self\.output\.push_str\(", "\);\s*\}\s*self\.visit_expr\(&arg\.node\)\?;\s*\}\s*self\.output\.push_str\("\)"\);\s*\}'''

new_call_code = '''Expr::Call(callee, _, args) => {
                if let Expr::PropertyAccess(object, property) = &callee.node {
                    self.output.push_str("(");
                    self.visit_expr(&object.node)?;
                    self.output.push_str(")->");
                    self.output.push_str(property);
                    self.output.push_str("(");
                    self.visit_expr(&object.node)?;
                    if !args.is_empty() {
                        self.output.push_str(", ");
                    }
                } else if let Expr::Identifier(id) = &callee.node {
                    if id == "print" {
                        self.output.push_str("printf(");
                        if let Some(arg) = args.first() {
                            if let Expr::Literal(Literal::String(s)) = &arg.node {
                                write!(&mut self.output, "\\" {}\\n\\"", s).unwrap();
                            } else {
                                self.output.push_str("\"%ld\\n\", ");
                                self.visit_expr(&arg.node)?;
                            }
                        }
                        self.output.push_str(")");
                        return Ok(());
                    }
                    self.output.push_str(id);
                    self.output.push_str("(");
                } else {
                    self.visit_expr(&callee.node)?;
                    self.output.push_str("(");
                }

                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.visit_expr(&arg.node)?;
                }
                self.output.push_str(")");
            }'''

content = re.sub(call_regex, new_call_code, content, flags=re.DOTALL)

with open('compiler/codegen/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)
