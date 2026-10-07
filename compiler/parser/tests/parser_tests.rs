use ast::*;
use parser::Parser;

fn parse_ok(source: &str) -> Program {
    let mut parser = Parser::new(source);
    match parser.parse_program() {
        Ok(prog) => prog,
        Err(errs) => {
            panic!("Expected successful parse, got errors: {:?}", errs);
        }
    }
}

fn parse_err(source: &str) -> Vec<parser::error::ParseError> {
    let mut parser = Parser::new(source);
    match parser.parse_program() {
        Ok(_) => panic!("Expected parsing to fail on: {}", source),
        Err(errs) => errs,
    }
}

#[test]
fn test_functions_and_variables() {
    let prog = parse_ok("void test() { var x = 1; mut y = 2; var z; }");
    assert_eq!(prog.declarations.len(), 1);

    if let Decl::Function(method, _) = &prog.declarations[0].node {
        assert_eq!(method.name, "test");
        if let Stmt::Block(stmts) = &method.body.node {
            assert_eq!(stmts.len(), 3);

            if let Stmt::VarDecl {
                is_final,
                name,
                initializer,
                ..
            } = &stmts[0].node
            {
                assert_eq!(is_final, &true); // `var` implies final in parser AST
                assert_eq!(name, "x");
                assert!(initializer.is_some());
            } else {
                panic!("Expected VarDecl");
            }

            if let Stmt::VarDecl {
                is_final,
                name,
                initializer,
                ..
            } = &stmts[1].node
            {
                assert_eq!(is_final, &false); // `mut` is not final
                assert_eq!(name, "y");
                assert!(initializer.is_some());
            } else {
                panic!("Expected VarDecl");
            }
        } else {
            panic!("Expected Block");
        }
    } else {
        panic!("Expected Function");
    }
}

#[test]
fn test_expressions() {
    // Note: * and / are currently missing from the parser's expression precedence rules.
    // Testing only supported operators here.
    let prog = parse_ok("void test() { 1 + 2 - 3 == 7; obj.prop ?. safe ?? fallback; arr[0]; }");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn test_classes_and_generics() {
    let prog = parse_ok(
        "
        class Box<T> extends Base implements IA, IB {
            T value;
            T get_value() { return this.value; }
        }
    ",
    );
    assert_eq!(prog.declarations.len(), 1);
    if let Decl::Class {
        name,
        type_params,
        extends_class,
        implements_interfaces,
        fields,
        methods,
        ..
    } = &prog.declarations[0].node
    {
        assert_eq!(name, "Box");
        assert_eq!(type_params, &vec!["T".to_string()]);
        assert_eq!(extends_class, &Some("Base".to_string()));
        assert_eq!(
            implements_interfaces,
            &vec!["IA".to_string(), "IB".to_string()]
        );
        assert_eq!(fields.len(), 1);
        assert_eq!(methods.len(), 1);
    } else {
        panic!("Expected Class");
    }
}

#[test]
fn test_loops() {
    let prog = parse_ok(
        "
        void test() {
            while (true) { }
            for i in 0..10 { }
            for item in arr { }
        }
    ",
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn test_imports_and_aliases() {
    let prog = parse_ok(
        "
        import { A, B as C } from \"module\";
        export type Id = String;
    ",
    );
    assert_eq!(prog.declarations.len(), 2);
    if let Decl::Import { path, items } = &prog.declarations[0].node {
        assert_eq!(path, "module");
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected Import");
    }

    if let Decl::TypeAlias {
        name, is_exported, ..
    } = &prog.declarations[1].node
    {
        assert_eq!(name, "Id");
        assert_eq!(is_exported, &true);
    } else {
        panic!("Expected TypeAlias");
    }
}

#[test]
fn test_option_result() {
    // Using in class fields because local variables don't support explicit type annotations in the parser yet.
    let prog = parse_ok(
        "
        class Test {
            Result<int> res;
            int? opt;
            int! bang; 
        }
    ",
    );
    assert_eq!(prog.declarations.len(), 1);
    if let Decl::Class { fields, .. } = &prog.declarations[0].node {
        assert_eq!(fields.len(), 3);
        assert_eq!(
            fields[1].field_type,
            Type::Option(Box::new(Type::Named("int".into(), vec![])))
        );
        assert_eq!(
            fields[2].field_type,
            Type::Result(Box::new(Type::Named("int".into(), vec![])))
        );
    }
}

#[test]
fn test_match_expression() {
    let _prog = parse_ok(
        "
        void test() {
            match obj {
                1 => 10,
                \"str\" => 20,
                _ => 30
            };
        }
    ",
    );
}

// NEGATIVE TESTS

#[test]
fn test_missing_delimiters() {
    let errs = parse_err("void test() { var x = 1 }"); // missing semicolon
    assert!(!errs.is_empty());
}

#[test]
fn test_incomplete_declaration() {
    let errs = parse_err("class { }"); // missing class name
    assert!(!errs.is_empty());
}

#[test]
fn test_malformed_expression() {
    let errs = parse_err("void test() { 1 + = 2; }"); // invalid binary
    assert!(!errs.is_empty());
}

#[test]
fn test_unexpected_eof() {
    let errs = parse_err("class Person {");
    assert!(!errs.is_empty());
}

#[test]
fn test_error_recovery() {
    // A missing semicolon on a variable declaration should not break the rest of the parsing completely,
    // though it might skip to the next synchronizable token.
    let errs = parse_err(
        "
        class A {
            int a // missing semicolon
            int b;
        }
    ",
    );
    assert!(!errs.is_empty());
}

#[test]
fn test_multiline_spans() {
    let source = "void test() {\n    var x = 1;\n}";
    let mut parser = Parser::new(source);
    let prog = parser.parse_program().unwrap();
    if let Decl::Function(method, _) = &prog.declarations[0].node {
        if let Stmt::Block(stmts) = &method.body.node {
            let stmt_span = stmts[0].span.clone();
            // It should be roughly 18..28 depending on exact byte positions
            assert!(stmt_span.start > 10);
            assert!(stmt_span.end > stmt_span.start);
        }
    }
}

// BUG REGRESSION TESTS

#[test]
fn test_bug_top_level_variables_unsupported() {
    // The parser expects only classes, functions, aliases, and imports at the top level.
    // `var x = 1;` at top level is interpreted as a function starting with an invalid type.
    let errs = parse_err("var x = 1;");
    assert!(!errs.is_empty());
    assert_eq!(errs[0].message, "Expected type name");
}
