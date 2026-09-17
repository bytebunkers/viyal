use parser::Parser;
use ast::*;

#[test]
fn test_simple_program() {
    let source = r#"
        class Person(String name, int age) {
            void greet() {
                print("Hello");
            }
        }
    "#;
    let mut parser = Parser::new(source);
    let program = parser.parse_program().expect("Failed to parse program");
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Decl::Class { name, primary_constructor, methods, extends_class, implements_interfaces, fields } = &program.declarations[0].node {
        assert_eq!(name, "Person");
        assert_eq!(primary_constructor.len(), 2);
        assert_eq!(primary_constructor[0].name, "name");
        assert_eq!(primary_constructor[0].param_type, Type::Named("String".to_string()));
        
        assert_eq!(primary_constructor[1].name, "age");
        assert_eq!(primary_constructor[1].param_type, Type::Named("int".to_string()));
        
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].name, "greet");
        assert_eq!(methods[0].return_type, Some(Type::Named("void".to_string())));
        
        if let Stmt::Block(stmts) = &methods[0].body.node {
            assert_eq!(stmts.len(), 1);
            if let Stmt::Expr(expr) = &stmts[0].node {
                if let Expr::Call(target, args) = &expr.node {
                    if let Expr::Identifier(id) = &target.node {
                        assert_eq!(id, "print");
                    } else {
                        panic!("Expected print identifier");
                    }
                    assert_eq!(args.len(), 1);
                    if let Expr::Literal(Literal::String(s)) = &args[0].node {
                        assert_eq!(s, "Hello");
                    } else {
                        panic!("Expected string literal");
                    }
                } else {
                    panic!("Expected call expression");
                }
            } else {
                panic!("Expected expression statement");
            }
        } else {
            panic!("Expected block statement");
        }
    } else {
        panic!("Expected class declaration");
    }
}
