use lexer::Lexer;
use parser::Parser;

#[test]
fn test_debug_read() {
    let source = "class Person { void greet() {} }\n";
    println!("File content bytes: {:?}", source.as_bytes());

    let mut lex = Lexer::new(source);
    while let Some(t) = lex.next() {
        println!("Token: {:?}", t);
    }

    let mut parser = Parser::new(source);
    let result = parser.parse_program();
    println!("Parse Result: {:?}", result);
}
