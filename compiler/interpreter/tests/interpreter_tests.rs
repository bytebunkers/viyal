use parser::Parser;
use interpreter::Interpreter;

#[test]
fn test_print_execution() {
    let source = r#"
        class Person(String name, int age) {
            void greet() {
                print("Hello");
            }
        }
    "#;
    
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    
    let mut output = String::new();
    let mut interpreter = Interpreter::new(&mut output);
    
    interpreter.interpret(&program).unwrap();
    
    assert_eq!(output, "Hello\n");
}
