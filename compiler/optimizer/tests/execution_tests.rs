use ast::{BinaryOp, Literal, Type};
use backend::execute_jit;
use mir::ir::{
    BasicBlock, Local, LocalDecl, MirFunction, MirProgram, Operand, Rvalue, Statement, Terminator,
};
use optimizer::optimize;
use std::collections::HashMap;

fn make_complex_math_program() -> MirProgram {
    let mut program = MirProgram::default();

    let mut func = MirFunction::new(
        "main".to_string(),
        Some(Type::Named("int".to_string(), vec![])),
    );

    // _0 = return value
    func.locals.push(LocalDecl {
        ty: Type::Named("int".to_string(), vec![]),
        name: Some("_0".to_string()),
        is_mut: true,
    });

    // _1 = 10
    func.locals.push(LocalDecl {
        ty: Type::Named("int".to_string(), vec![]),
        name: Some("_1".to_string()),
        is_mut: true,
    });

    // _2 = 20
    func.locals.push(LocalDecl {
        ty: Type::Named("int".to_string(), vec![]),
        name: Some("_2".to_string()),
        is_mut: true,
    });

    // _3 = _1 + _2
    func.locals.push(LocalDecl {
        ty: Type::Named("int".to_string(), vec![]),
        name: Some("_3".to_string()),
        is_mut: true,
    });

    let bb = BasicBlock {
        id: 0,
        phis: vec![],
        statements: vec![
            Statement::Assign(
                Local(1),
                Rvalue::Use(Operand::Constant(Literal::Integer(10))),
            ),
            Statement::Assign(
                Local(2),
                Rvalue::Use(Operand::Constant(Literal::Integer(20))),
            ),
            Statement::Assign(
                Local(3),
                Rvalue::BinaryOp(
                    BinaryOp::Add,
                    Operand::Copy(Local(1)),
                    Operand::Copy(Local(2)),
                ),
            ),
            Statement::Assign(Local(0), Rvalue::Use(Operand::Copy(Local(3)))),
        ],
        terminator: Terminator::Return {
            value: Some(Operand::Copy(Local(0))),
        },
    };

    func.basic_blocks.push(bb);
    program.functions.insert("main".to_string(), func);
    program
}

#[test]
fn test_execution_unoptimized_vs_optimized() {
    let unopt_prog = make_complex_math_program();
    let opt_prog = optimize(unopt_prog.clone());

    let unopt_result = execute_jit(&unopt_prog).expect("Unoptimized JIT failed");
    let opt_result = execute_jit(&opt_prog).expect("Optimized JIT failed");

    assert_eq!(unopt_result, 30, "Unoptimized result incorrect");
    assert_eq!(
        unopt_result, opt_result,
        "Optimized execution result differs from unoptimized"
    );
}
