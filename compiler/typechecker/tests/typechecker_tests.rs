use parser::Parser;
use typechecker::TypeChecker;

fn check_ok(source: &str) {
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    let mut typechecker = TypeChecker::new();
    let result = typechecker.check_program(&program);
    assert!(result.is_ok(), "Expected OK, got {:?}", result);
}

fn check_err(source: &str, expected_msg: &str) {
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    let mut typechecker = TypeChecker::new();
    let result = typechecker.check_program(&program);
    assert!(result.is_err(), "Expected error for: {}", source);
    let err = result.unwrap_err();
    assert!(
        err.message.contains(expected_msg),
        "Expected error containing '{}', got '{}'",
        expected_msg,
        err.message
    );
}

#[test]
fn test_valid_program() {
    check_ok(
        r#"
        class Person(String name, int age) {
            void greet() {
                print("Hello");
                print(name);
                print(age);
            }
        }
    "#,
    );
}

#[test]
fn test_undefined_variable() {
    check_err(
        r#"
        class Person(String name) {
            void greet() {
                print(age); // 'age' is not defined
            }
        }
    "#,
        "Undefined variable 'age'",
    );
}

#[test]
fn test_assignment_compatibility() {
    check_ok(
        r#"
        void test() {
            mut x = 1;
            x = 2; // Ok
        }
    "#,
    );

    check_err(
        r#"
        void test() {
            mut x = 1;
            x = "hello";
        }
    "#,
        "Type mismatch in binary operation Assign",
    );

    check_err(
        r#"
        void test() {
            var x = 1;
            x = 2;
        }
    "#,
        "Cannot reassign immutable variable",
    );
}

#[test]
fn test_function_return_types() {
    check_ok(
        r#"
        int get_age() {
            return 42;
        }
    "#,
    );

    check_err(
        r#"
        int get_name() {
            return "Alice";
        }
    "#,
        "Return type mismatch",
    );
}

#[test]
fn test_function_arguments() {
    check_ok(
        r#"
        void greet(String name, int age) {}
        void test() {
            greet("Bob", 30);
        }
    "#,
    );

    check_err(
        r#"
        void greet(String name, int age) {}
        void test() {
            greet("Bob", "thirty"); // Wrong type
        }
    "#,
        "Argument 1 type mismatch",
    );

    check_err(
        r#"
        void greet(String name, int age) {}
        void test() {
            greet("Bob"); // Wrong count
        }
    "#,
        "expects 2 arguments, got 1",
    );
}

#[test]
fn test_operator_type() {
    check_ok(
        r#"
        void test() {
            var x = 1 + 2;
        }
    "#,
    );

    check_err(
        r#"
        void test() {
            var x = 1 + true;
        }
    "#,
        "Type mismatch in binary operation Add",
    );
}

#[test]
fn test_shadowing() {
    check_ok(
        r#"
        void test() {
            mut x = 1;
            {
                mut x = "hello"; // Shadowing in inner scope
            }
            x = 2; // Should still be int
        }
    "#,
    );
}
