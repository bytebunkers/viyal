use parser::Parser;
use typechecker::TypeChecker;

#[test]
fn test_valid_program() {
    let source = r#"
        class Person(String name, int age) {
            void greet() {
                print("Hello");
                print(name);
                print(age);
            }
        }
    "#;
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    
    let mut typechecker = TypeChecker::new();
    let result = typechecker.check_program(&program);
    assert!(result.is_ok());
}

#[test]
fn test_undefined_variable() {
    let source = r#"
        class Person(String name) {
            void greet() {
                print(age); // 'age' is not defined
            }
        }
    "#;
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    
    let mut typechecker = TypeChecker::new();
    let result = typechecker.check_program(&program);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "Undefined variable 'age'");
}
