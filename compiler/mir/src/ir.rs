use ast::{BinaryOp, Literal, Type};
use std::collections::HashMap;

/// A local variable or temporary in a MIR function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Local(pub usize);

/// Declaration of a local variable
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub ty: Type,
    pub name: Option<String>,
    pub is_mut: bool,
}

/// A basic block in the Control Flow Graph (CFG)
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub phis: Vec<Phi>,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// A Phi node for SSA form
#[derive(Debug, Clone)]
pub struct Phi {
    pub dest: Local,
    /// The original local this Phi node resolves
    pub orig_local: Local,
    /// (Operand, PredecessorBlockId)
    pub operands: Vec<(Operand, usize)>,
}


#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment of an Rvalue to a Local (e.g., _1 = 2 + _3)
    Assign(Local, Rvalue),
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    /// Simply read an operand
    Use(Operand),
    /// Perform a binary operation
    BinaryOp(BinaryOp, Operand, Operand),
    /// Function or method call
    Call {
        func: Operand,
        args: Vec<Operand>,
    },
    /// Property access (e.g., _1.field)
    PropertyAccess(Operand, String),
    /// Method call (e.g., _1.method(args))
    MethodCall(Operand, String, Vec<Operand>),
    /// Array initialization (e.g., [1, 2, 3])
    Array(Vec<Operand>),
    /// Map initialization (e.g., { "a": 1 })
    Map(Vec<(Operand, Operand)>),
    /// Length of array/string
    Length(Operand),
    /// Array/Map indexing (e.g., arr[i])
    Index(Operand, Operand),
    /// Try operator (e.g., try expr)
    Try(Operand),
    /// Object allocation (e.g., new Class(args))
    New(String, Vec<Operand>),
}

#[derive(Debug, Clone)]
pub enum Operand {
    /// A constant value
    Constant(Literal),
    /// Reading from a local variable
    Copy(Local),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional jump to another block
    Goto { target: usize },
    /// Conditional branch
    If {
        cond: Operand,
        then_target: usize,
        else_target: usize,
    },
    /// Branch on Result::Ok
    IfOk {
        val: Operand,
        then_target: usize,
        else_target: usize,
    },
    /// Return from the function
    Return { value: Option<Operand> },
    /// Unreachable code (e.g., after an infinite loop or panic)
    Unreachable,
}

/// A full MIR representation of a function or method
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub locals: Vec<LocalDecl>,
    pub basic_blocks: Vec<BasicBlock>,
    pub return_type: Option<Type>,
    pub params: Vec<Local>, // Indices into `locals`
}

impl MirFunction {
    pub fn new(name: String, return_type: Option<Type>) -> Self {
        Self {
            name,
            locals: Vec::new(),
            basic_blocks: Vec::new(),
            return_type,
            params: Vec::new(),
        }
    }
}

/// The entire MIR representation of a program
#[derive(Debug, Clone, Default)]
pub struct MirProgram {
    pub functions: HashMap<String, MirFunction>,
}
