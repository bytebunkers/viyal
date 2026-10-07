use lexer::Lexer;
use lexer::token::Token;

fn assert_tokens(source: &str, expected: Vec<Token>) {
    let mut lexer = Lexer::new(source);
    for exp in expected {
        let tok = lexer
            .next()
            .expect("Expected token, got EOF")
            .expect("Expected valid token, got error");
        assert_eq!(tok.token, exp);
    }
    assert!(
        lexer.next().is_none(),
        "Expected EOF, but found more tokens"
    );
}

#[test]
fn test_keywords() {
    let source = "class extends implements new this super interface void final var mut if else for while do switch return async await true false null in type match import export from as int double bool String";
    assert_tokens(
        source,
        vec![
            Token::Class,
            Token::Extends,
            Token::Implements,
            Token::New,
            Token::This,
            Token::Super,
            Token::Interface,
            Token::Void,
            Token::Final,
            Token::Var,
            Token::Mut,
            Token::If,
            Token::Else,
            Token::For,
            Token::While,
            Token::Do,
            Token::Switch,
            Token::Return,
            Token::Async,
            Token::Await,
            Token::True,
            Token::False,
            Token::Null,
            Token::In,
            Token::TypeKeyword,
            Token::Match,
            Token::Import,
            Token::Export,
            Token::From,
            Token::As,
            Token::Int,
            Token::Double,
            Token::BoolType,
            Token::StringType,
        ],
    );
}

#[test]
fn test_identifiers() {
    let source = "valid_ident _alsoValid var123 camelCase PascalCase";
    assert_tokens(
        source,
        vec![
            Token::Identifier("valid_ident".into()),
            Token::Identifier("_alsoValid".into()),
            Token::Identifier("var123".into()),
            Token::Identifier("camelCase".into()),
            Token::Identifier("PascalCase".into()),
        ],
    );
}

#[test]
fn test_integers_and_floats() {
    let source = "0 12345 0.0 3.14159";
    assert_tokens(
        source,
        vec![
            Token::Integer(0),
            Token::Integer(12345),
            Token::Float(0.0),
            Token::Float(3.14159),
        ],
    );
}

#[test]
fn test_strings_and_escapes() {
    let source = r#" "hello" "with \" escape" "" "#;
    assert_tokens(
        source,
        vec![
            Token::StringLit("hello".into()),
            Token::StringLit(r#"with \" escape"#.into()), // Note: the lexer doesn't unescape, it just captures the raw content
            Token::StringLit("".into()),
        ],
    );
}

#[test]
fn test_operators_and_punctuation() {
    let source = "+ - * / == != < > <= >= = => ? ! ?? ?. or ( ) { } [ ] ; , . .. :";
    assert_tokens(
        source,
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::EqEq,
            Token::NotEq,
            Token::Less,
            Token::Greater,
            Token::LessEq,
            Token::GreaterEq,
            Token::Eq,
            Token::FatArrow,
            Token::Question,
            Token::Bang,
            Token::DoubleQuestion,
            Token::QuestionDot,
            Token::Or,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
            Token::Semi,
            Token::Comma,
            Token::Dot,
            Token::DotDot,
            Token::Colon,
        ],
    );
}

#[test]
fn test_comments() {
    let source = "
    // single line comment
    var x = 1; // end of line comment
    /* multi line
       block comment */
    var y = 2;
    ";
    assert_tokens(
        source,
        vec![
            Token::Var,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::Integer(1),
            Token::Semi,
            Token::Var,
            Token::Identifier("y".into()),
            Token::Eq,
            Token::Integer(2),
            Token::Semi,
        ],
    );
}

#[test]
fn test_spans_and_multiline() {
    let source = "var x\n  = 10;";
    let mut lexer = Lexer::new(source);

    let t1 = lexer.next().unwrap().unwrap();
    assert_eq!(t1.token, Token::Var);
    assert_eq!(t1.span, 0..3);

    let t2 = lexer.next().unwrap().unwrap();
    assert_eq!(t2.token, Token::Identifier("x".into()));
    assert_eq!(t2.span, 4..5);

    let t3 = lexer.next().unwrap().unwrap();
    assert_eq!(t3.token, Token::Eq);
    assert_eq!(t3.span, 8..9); // 5 is \n, 6-7 are spaces

    let t4 = lexer.next().unwrap().unwrap();
    assert_eq!(t4.token, Token::Integer(10));
    assert_eq!(t4.span, 10..12);

    let t5 = lexer.next().unwrap().unwrap();
    assert_eq!(t5.token, Token::Semi);
    assert_eq!(t5.span, 12..13);

    assert!(lexer.next().is_none());
}

#[test]
fn test_invalid_characters() {
    let source = "var @ x = 1;";
    let mut lexer = Lexer::new(source);

    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Var);
    // `@` is invalid, Logos will emit an error
    let err = lexer.next().unwrap();
    assert!(err.is_err());

    // Recovery: should continue lexing
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::Identifier("x".into())
    );
}

#[test]
fn test_unicode_handling() {
    // Current identifier regex is ASCII only. Unicode in identifiers will cause lexer errors.
    // Unicode inside strings should work fine.
    let source = r#" "hello 世界" 变量 "#;
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::StringLit("hello 世界".into())
    );

    // 变量 will trigger an error since they aren't [a-zA-Z_]
    let err = lexer.next().unwrap();
    assert!(err.is_err());
}

#[test]
fn test_eof() {
    let source = "";
    let mut lexer = Lexer::new(source);
    assert!(lexer.next().is_none());
}
