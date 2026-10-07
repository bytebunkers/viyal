use lexer::Lexer;

#[test]
fn test_dump() {
    let source = "class Person { void greet() {} }\n";
    let mut lex = Lexer::new(source);
    while let Some(t) = lex.next() {
        println!("{:?}", t);
    }
}
