use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f\r]+")] // Ignore whitespace
#[logos(skip r"//[^\n]*")] // Ignore single-line comments
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")] // Ignore block comments
pub enum Token {
    // Keywords
    #[token("class")] Class,
    #[token("extends")] Extends,
    #[token("implements")] Implements,
    #[token("new")] New,
    #[token("this")] This,
    #[token("super")] Super,
    #[token("interface")] Interface,
    #[token("void")] Void,
    #[token("final")] Final,
    #[token("var")] Var,
    #[token("if")] If,
    #[token("else")] Else,
    #[token("for")] For,
    #[token("while")] While,
    #[token("do")] Do,
    #[token("switch")] Switch,
    #[token("return")] Return,
    #[token("async")] Async,
    #[token("await")] Await,
    #[token("true")] True,
    #[token("false")] False,
    #[token("null")] Null,
    #[token("mut")] Mut,
    #[token("in")] In,
    #[token("type")] TypeKeyword,
    #[token("match")] Match,
    #[token("import")] Import,
    #[token("export")] Export,
    #[token("from")] From,
    #[token("as")] As,

    // Types (built-ins for now)
    #[token("int")] Int,
    #[token("double")] Double,
    #[token("bool")] BoolType,
    #[token("String")] StringType,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    Float(f64),

    // String literals (basic support, no interpolation yet)
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLit(String),

    // Operators
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("==")] EqEq,
    #[token("!=")] NotEq,
    #[token("<")] Less,
    #[token(">")] Greater,
    #[token("<=")] LessEq,
    #[token(">=")] GreaterEq,
    #[token("=")] Eq,
    #[token("=>")] FatArrow,
    #[token("?")] Question,
    #[token("!")] Bang,
    #[token("??")] DoubleQuestion,
    #[token("?.")] QuestionDot,
    #[token("or")] Or,
    
    // Punctuation
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token(";")] Semi,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token("..")] DotDot,
    #[token(":")] Colon,
}
