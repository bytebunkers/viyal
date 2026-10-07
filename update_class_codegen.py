import re

with open('compiler/codegen/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

class_regex = r'''Decl::Class \{ name, fields, methods, \.\. \} => \{
                // Generate struct definition
                writeln!\(&mut self\.output, "struct \{\} \{\{", name\)\.unwrap\(\);
                for field in fields \{
                    let field_c_type = self\.type_to_c\(&field\.field_type\);
                    writeln!\(&mut self\.output, "    \{\} \{\};", field_c_type, field\.name\)\.unwrap\(\);
                \}
                self\.output\.push_str\("\};\n\n"\);

                for method in methods \{
                    let ret_type = method\.return_type\.as_ref\(\)\.map_or\("void"\.to_string\(\), \|t\| self\.type_to_c\(t\)\);
                    
                    if name == "Main" && method\.name == "run" \{
                        self\.output\.push_str\("int main\(\) \{\n"\);
                        self\.indent_level \+= 1;
                        self\.visit_stmt\(&method\.body\.node\)\?;
                        self\.push_indent\(\);
                        self\.output\.push_str\("return 0;\n"\);
                        self\.indent_level -= 1;
                        self\.output\.push_str\("\}\n\n"\);
                    \} else \{
                        // Generate flattened method name taking `this` pointer
                        write!\(&mut self\.output, "\{\} \{\}_\{\}\(struct \{\}\* this", ret_type, name, method\.name, name\)\.unwrap\(\);
                        for param in &method\.params \{
                            let param_c_type = self\.type_to_c\(&param\.param_type\);
                            write!\(&mut self\.output, ", \{\} \{\}", param_c_type, param\.name\)\.unwrap\(\);
                        \}
                        self\.output\.push_str\("\) \{\n"\);
                        self\.indent_level \+= 1;
                        self\.visit_stmt\(&method\.body\.node\)\?;
                        self\.indent_level -= 1;
                        self\.output\.push_str\("\}\n\n"\);
                    \}
                \}
            \}'''

new_class_code = '''Decl::Class { name, fields, methods, .. } => {
                // Forward declare the struct so function pointers can use it
                writeln!(&mut self.output, "struct {};", name).unwrap();
                
                // Generate struct definition
                writeln!(&mut self.output, "struct {} {{", name).unwrap();
                for field in fields {
                    let field_c_type = self.type_to_c(&field.field_type);
                    writeln!(&mut self.output, "    {} {};", field_c_type, field.name).unwrap();
                }
                
                // Generate function pointers for methods
                for method in methods {
                    if name == "Main" && method.name == "run" { continue; }
                    let ret_type = method.return_type.as_ref().map_or("void".to_string(), |t| self.type_to_c(t));
                    write!(&mut self.output, "    {} (*{})(struct {}* this", ret_type, method.name, name).unwrap();
                    for param in &method.params {
                        let param_c_type = self.type_to_c(&param.param_type);
                        write!(&mut self.output, ", {}", param_c_type).unwrap();
                    }
                    self.output.push_str(");\n");
                }
                self.output.push_str("};\n\n");

                for method in methods {
                    let ret_type = method.return_type.as_ref().map_or("void".to_string(), |t| self.type_to_c(t));
                    
                    if name == "Main" && method.name == "run" {
                        self.output.push_str("int main() {\n");
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.push_indent();
                        self.output.push_str("return 0;\n");
                        self.indent_level -= 1;
                        self.output.push_str("}\n\n");
                    } else {
                        // Generate flattened method name taking `this` pointer
                        write!(&mut self.output, "{} {}_impl_{}(struct {}* this", ret_type, name, method.name, name).unwrap();
                        for param in &method.params {
                            let param_c_type = self.type_to_c(&param.param_type);
                            write!(&mut self.output, ", {} {}", param_c_type, param.name).unwrap();
                        }
                        self.output.push_str(") {\n");
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.indent_level -= 1;
                        self.output.push_str("}\n\n");
                    }
                }
                
                // Generate Constructor Allocator (_new)
                if name != "Main" {
                    writeln!(&mut self.output, "struct {0}* {0}_new() {{", name).unwrap();
                    writeln!(&mut self.output, "    struct {0}* obj = calloc(1, sizeof(struct {0}));", name).unwrap();
                    for method in methods {
                        if method.name == "run" { continue; }
                        writeln!(&mut self.output, "    obj->{1} = {0}_impl_{1};", name, method.name).unwrap();
                    }
                    writeln!(&mut self.output, "    return obj;\n}}\n").unwrap();
                }
            }'''

content = re.sub(class_regex, new_class_code, content, flags=re.DOTALL)

with open('compiler/codegen/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)
