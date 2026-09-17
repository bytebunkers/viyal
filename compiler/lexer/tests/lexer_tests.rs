use lexer::Lexer;
use lexer::token::Token;

#[test]
fn test_keywords() {
    let source = "class interface void final var if else for while do switch return async await true false null";
    let mut lexer = Lexer::new(source);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Class);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Interface);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Void);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Final);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Var);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::If);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Else);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::For);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::While);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Do);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Switch);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Return);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Async);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Await);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::True);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::False);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Null);
    assert!(lexer.next().is_none());
}

#[test]
fn test_identifiers_and_literals() {
    let source = r#"my_var 123 45.67 "hello world""#;
    let mut lexer = Lexer::new(source);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("my_var".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Integer(123));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Float(45.67));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::StringLit("hello world".to_string()));
    assert!(lexer.next().is_none());
}

#[test]
fn test_operators_and_punctuation() {
    let source = "+ - * / == != < > <= >= = => ? ( ) { } [ ] ; , .";
    let mut lexer = Lexer::new(source);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Plus);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Minus);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Star);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Slash);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::EqEq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::NotEq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Less);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Greater);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LessEq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::GreaterEq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Eq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::FatArrow);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Question);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LBrace);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RBrace);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LBracket);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RBracket);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Semi);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Comma);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Dot);
    assert!(lexer.next().is_none());
}

#[test]
fn test_comments() {
    let source = "
    // This is a single line comment
    int x = 10;
    /* This is a 
       multi-line block comment */
    double y = 20.0;
    ";
    let mut lexer = Lexer::new(source);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Int);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("x".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Eq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Integer(10));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Semi);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Double);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("y".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Eq);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Float(20.0));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Semi);
    
    assert!(lexer.next().is_none());
}

#[test]
fn test_simple_program() {
    let source = r#"
        class Person(String name, int age) {
            void greet() {
                print("Hello");
            }
        }
    "#;
    let mut lexer = Lexer::new(source);
    
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Class);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("Person".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::StringType);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("name".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Comma);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Int);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("age".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LBrace);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Void);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("greet".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LBrace);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier("print".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::StringLit("Hello".to_string()));
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RParen);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Semi);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RBrace);
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::RBrace);
    assert!(lexer.next().is_none());
}
