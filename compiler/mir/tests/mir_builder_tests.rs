use ast::Program;
use mir::builder::MirBuilder;
use mir::ir::{Operand, Rvalue, Statement, Terminator};
use parser::Parser;

fn build_mir(source: &str) -> mir::ir::MirProgram {
    let mut parser = Parser::new(source);
    let program = parser.parse_program().unwrap();
    let builder = MirBuilder::new();
    builder.build(&program)
}

#[test]
fn test_mir_empty_function() {
    let mir = build_mir("void test() {}");
    assert_eq!(mir.functions.len(), 1);
    let func = &mir.functions["test"];
    assert_eq!(func.name, "test");
    assert_eq!(func.basic_blocks.len(), 1);
    let start_block = &func.basic_blocks[0];
    assert!(matches!(
        start_block.terminator,
        Terminator::Return { value: None }
    ));
}

#[test]
fn test_mir_var_decl() {
    let mir = build_mir("void test() { var x = 42; }");
    let func = &mir.functions["test"];
    assert_eq!(func.locals.len(), 1);
    assert_eq!(func.locals[0].name.as_deref(), Some("x"));

    let block = &func.basic_blocks[0];
    assert_eq!(block.statements.len(), 1);
    match &block.statements[0] {
        Statement::Assign(local, Rvalue::Use(Operand::Constant(lit))) => {
            assert_eq!(local.0, 0); // local index 0
            if let ast::Literal::Integer(val) = lit {
                assert_eq!(*val, 42);
            } else {
                panic!("Expected integer literal");
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_mir_if_statement() {
    let mir = build_mir("void test() { if (true) { return; } else { return; } }");
    let func = &mir.functions["test"];

    // Should have: entry, then, else, merge blocks. (Wait, Return creates new block too!)
    // Let's just check there's an If terminator in the first block.
    let start_block = &func.basic_blocks[0];
    match &start_block.terminator {
        Terminator::If {
            cond: Operand::Constant(ast::Literal::Boolean(true)),
            then_target,
            else_target,
        } => {
            assert_eq!(*then_target, 1);
            assert_eq!(*else_target, 2);
        }
        _ => panic!("Expected If terminator"),
    }
}

#[test]
fn test_mir_while_statement() {
    let mir = build_mir("void test() { while (true) {} }");
    let func = &mir.functions["test"];

    let start_block = &func.basic_blocks[0];
    // Start block just gotos to cond block
    match &start_block.terminator {
        Terminator::Goto { target } => {
            assert_eq!(*target, 1);
        }
        _ => panic!("Expected Goto terminator"),
    }

    // Cond block
    let cond_block = &func.basic_blocks[1];
    match &cond_block.terminator {
        Terminator::If {
            then_target,
            else_target,
            ..
        } => {
            assert_eq!(*then_target, 2);
            assert_eq!(*else_target, 3);
        }
        _ => panic!("Expected If terminator in cond block"),
    }
}

#[test]
fn test_mir_for_range() {
    let mir = build_mir("void test() { for i in 0..10 {} }");
    let func = &mir.functions["test"];

    // Locals: index (i), __end, cond_local, incremented_local
    assert!(func.locals.len() >= 4);
    assert_eq!(func.locals[0].name.as_deref(), Some("i"));
    assert_eq!(func.locals[1].name.as_deref(), Some("__end"));
}

#[test]
fn test_mir_return_expr() {
    let mir = build_mir("int test() { return 100; }");
    let func = &mir.functions["test"];
    let start_block = &func.basic_blocks[0];
    match &start_block.terminator {
        Terminator::Return {
            value: Some(Operand::Constant(ast::Literal::Integer(100))),
        } => {}
        _ => panic!("Expected Return terminator with value 100"),
    }
}
