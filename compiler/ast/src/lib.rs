use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary(Box<Spanned<Expr>>, BinaryOp, Box<Spanned<Expr>>),
    Call(Box<Spanned<Expr>>, Vec<Type>, Vec<Spanned<Expr>>),
    PropertyAccess(Box<Spanned<Expr>>, String),
    SafePropertyAccess(Box<Spanned<Expr>>, String),
    PropertyAssign(Box<Spanned<Expr>>, String, Box<Spanned<Expr>>),
    NullCoalesce(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    New(String, Vec<Type>, Vec<Spanned<Expr>>),
    Array(Vec<Spanned<Expr>>),
    Index(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    IndexAssign(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Map(Vec<(Spanned<Expr>, Spanned<Expr>)>),
    Try(Box<Spanned<Expr>>),
    UnwrapOrElse(Box<Spanned<Expr>>, Box<Spanned<Stmt>>),
    This,
    Super,
    Match(Box<Spanned<Expr>>, Vec<(MatchPattern, Spanned<Expr>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MatchPattern {
    Literal(Literal),
    Identifier(String), // Variable binding or enum variant
    CatchAll,           // _
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Assign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Spanned<Expr>),
    VarDecl {
        is_final: bool,
        type_annot: Option<Type>,
        name: String,
        initializer: Option<Spanned<Expr>>,
    },
    Block(Vec<Spanned<Stmt>>),
    If {
        condition: Spanned<Expr>,
        then_branch: Box<Spanned<Stmt>>,
        else_branch: Option<Box<Spanned<Stmt>>>,
    },
    While {
        condition: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    ForIn {
        item_name: String,
        iterable: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    ForRange {
        item_name: String,
        start: Spanned<Expr>,
        end: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    Return(Option<Spanned<Expr>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Class {
        name: String,
        type_params: Vec<String>,
        extends_class: Option<String>,
        implements_interfaces: Vec<String>,
        fields: Vec<Field>,
        primary_constructor: Vec<Param>,
        methods: Vec<Method>,
        is_exported: bool,
    },
    Function(Method, bool), // (Method, is_exported)
    TypeAlias {
        name: String,
        target_type: Type,
        is_exported: bool,
    },
    Import {
        path: String,
        items: Vec<(String, Option<String>)>, // (name, as alias)
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub is_final: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method {
    pub return_type: Option<Type>,
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub body: Spanned<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Named(String, Vec<Type>),
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Nullable(Box<Type>),
    Result(Box<Type>),
    Option(Box<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Result(inner) => write!(f, "Result<{}>", inner),
            Type::Option(inner) => write!(f, "Option<{}>", inner),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Vec<Spanned<Decl>>,
}
