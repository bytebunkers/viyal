use ast::{BinaryOp, Literal, Type};
use mir::ir::{
    BasicBlock, Local, LocalDecl, MirFunction, MirProgram, Operand, Phi, Rvalue, Statement,
    Terminator,
};
use optimizer::optimize;
use std::collections::HashMap;

fn new_func() -> MirFunction {
    MirFunction {
        name: "test".to_string(),
        params: vec![],
        return_type: None,
        locals: vec![],
        basic_blocks: vec![],
    }
}

fn add_local(func: &mut MirFunction, name: &str) -> Local {
    let id = func.locals.len();
    func.locals.push(LocalDecl {
        ty: Type::Named("Any".to_string(), vec![]),
        name: Some(name.to_string()),
        is_mut: true,
    });
    Local(id)
}

fn add_block(func: &mut MirFunction) -> usize {
    let id = func.basic_blocks.len();
    func.basic_blocks.push(BasicBlock {
        id,
        phis: vec![],
        statements: vec![],
        terminator: Terminator::Unreachable,
    });
    id
}

#[test]
fn test_sccp_constant_propagation() {
    let mut func = new_func();
    let l_const1 = add_local(&mut func, "c1"); // 0
    let l_const2 = add_local(&mut func, "c2"); // 1
    let l_sum = add_local(&mut func, "sum"); // 2

    let b0 = add_block(&mut func);

    // c1 = 10
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_const1,
        Rvalue::Use(Operand::Constant(Literal::Integer(10))),
    ));
    // c2 = 20
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_const2,
        Rvalue::Use(Operand::Constant(Literal::Integer(20))),
    ));
    // sum = c1 + c2
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_sum,
        Rvalue::BinaryOp(
            BinaryOp::Add,
            Operand::Copy(l_const1),
            Operand::Copy(l_const2),
        ),
    ));
    func.basic_blocks[b0].terminator = Terminator::Return {
        value: Some(Operand::Copy(l_sum)),
    };

    let mut prog = MirProgram {
        functions: HashMap::new(),
        classes: HashMap::new(),
    };
    prog.functions.insert("test".to_string(), func);

    let mut opt_prog = optimize(prog);
    let opt_func = opt_prog.functions.get_mut("test").unwrap();

    let block = &opt_func.basic_blocks[0];

    // We expect the third statement to have been constant folded to Use(Constant(30))
    // Or rather, SCCP replaces uses with constants, and then evaluates operations.
    let mut found_30 = false;
    println!("{:#?}", block.statements);
    for stmt in &block.statements {
        if let Statement::Assign(_, rvalue) = stmt {
            if let Rvalue::Use(Operand::Constant(Literal::Integer(30))) = rvalue {
                found_30 = true;
            }
        }
    }
    assert!(
        found_30,
        "Constant propagation failed to fold 10 + 20 to 30"
    );
}

#[test]
fn test_dce_unreachable_blocks() {
    let mut func = new_func();
    let l_cond = add_local(&mut func, "cond"); // 0
    let b0 = add_block(&mut func); // entry
    let b1 = add_block(&mut func); // then
    let b2 = add_block(&mut func); // else (unreachable because cond is true)

    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_cond,
        Rvalue::Use(Operand::Constant(Literal::Boolean(true))),
    ));
    func.basic_blocks[b0].terminator = Terminator::If {
        cond: Operand::Copy(l_cond),
        then_target: b1,
        else_target: b2,
    };

    func.basic_blocks[b1].terminator = Terminator::Return { value: None };
    func.basic_blocks[b2].terminator = Terminator::Return { value: None };

    let mut prog = MirProgram {
        functions: HashMap::new(),
        classes: HashMap::new(),
    };
    prog.functions.insert("test".to_string(), func);

    let mut opt_prog = optimize(prog);
    let opt_func = opt_prog.functions.get_mut("test").unwrap();

    // SCCP should fold the branch, DCE should remove b2.
    assert_eq!(
        opt_func.basic_blocks.len(),
        2,
        "DCE failed to remove unreachable block"
    );

    // Check that b0 terminator is now Goto(b1)
    match opt_func.basic_blocks[0].terminator {
        Terminator::Goto { target } => assert_eq!(target, 1),
        _ => panic!("Expected Goto terminator"),
    }
}

#[test]
fn test_gvn_common_subexpression_elimination() {
    let mut func = new_func();
    let l_x = add_local(&mut func, "x"); // 0
    let l_y = add_local(&mut func, "y"); // 1
    let l_z = add_local(&mut func, "z"); // 2

    let b0 = add_block(&mut func);

    // x = 10 + 20
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_x,
        Rvalue::BinaryOp(
            BinaryOp::Add,
            Operand::Constant(Literal::Integer(10)),
            Operand::Constant(Literal::Integer(20)),
        ),
    ));
    // y = 10 + 20
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_y,
        Rvalue::BinaryOp(
            BinaryOp::Add,
            Operand::Constant(Literal::Integer(10)),
            Operand::Constant(Literal::Integer(20)),
        ),
    ));
    func.basic_blocks[b0].terminator = Terminator::Return { value: None };

    let mut prog = MirProgram {
        functions: HashMap::new(),
        classes: HashMap::new(),
    };
    prog.functions.insert("test".to_string(), func);

    // To only test GVN, we can't easily isolate it if SCCP also folds 10+20.
    // Let's use Top variables instead.

    let mut func2 = new_func();
    let l_a = add_local(&mut func2, "a"); // 0
    let l_b = add_local(&mut func2, "b"); // 1
    let l_x = add_local(&mut func2, "x"); // 2
    let l_y = add_local(&mut func2, "y"); // 3

    let b0 = add_block(&mut func2);

    // x = a + b
    func2.basic_blocks[b0].statements.push(Statement::Assign(
        l_x,
        Rvalue::BinaryOp(BinaryOp::Add, Operand::Copy(l_a), Operand::Copy(l_b)),
    ));
    // y = a + b
    func2.basic_blocks[b0].statements.push(Statement::Assign(
        l_y,
        Rvalue::BinaryOp(BinaryOp::Add, Operand::Copy(l_a), Operand::Copy(l_b)),
    ));
    func2.basic_blocks[b0].terminator = Terminator::Return { value: None };

    let mut prog2 = MirProgram {
        functions: HashMap::new(),
        classes: HashMap::new(),
    };
    prog2.functions.insert("test".to_string(), func2);

    let mut opt_prog2 = optimize(prog2);
    let opt_func2 = opt_prog2.functions.get_mut("test").unwrap();
    let block = &opt_func2.basic_blocks[0];
    println!("{:#?}", block.statements);
    // The second assignment should be changed by GVN to Use(Operand::Copy(some_local))
    let mut found_use = false;
    for stmt in &block.statements {
        if let Statement::Assign(_, Rvalue::Use(Operand::Copy(_))) = stmt {
            found_use = true;
        }
    }
    assert!(found_use, "GVN failed to eliminate common subexpression");
}

#[test]
fn test_sccp_division_by_zero() {
    let mut func = new_func();
    let l_c1 = add_local(&mut func, "c1"); // 0
    let l_c2 = add_local(&mut func, "c2"); // 1
    let l_div = add_local(&mut func, "div"); // 2

    let b0 = add_block(&mut func);

    // c1 = 10
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_c1,
        Rvalue::Use(Operand::Constant(Literal::Integer(10))),
    ));
    // c2 = 0
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_c2,
        Rvalue::Use(Operand::Constant(Literal::Integer(0))),
    ));
    // div = c1 / c2
    func.basic_blocks[b0].statements.push(Statement::Assign(
        l_div,
        Rvalue::BinaryOp(BinaryOp::Div, Operand::Copy(l_c1), Operand::Copy(l_c2)),
    ));
    func.basic_blocks[b0].terminator = Terminator::Return {
        value: Some(Operand::Copy(l_div)),
    };

    let mut prog = MirProgram {
        functions: HashMap::new(),
        classes: HashMap::new(),
    };
    prog.functions.insert("test".to_string(), func);

    let mut opt_prog = optimize(prog);
    let opt_func = opt_prog.functions.get_mut("test").unwrap();

    let block = &opt_func.basic_blocks[0];

    // Div by 0 should NOT be folded (it should evaluate to Top, remaining a Div operation)
    let mut has_div = false;
    for stmt in &block.statements {
        if let Statement::Assign(_, rvalue) = stmt {
            if let Rvalue::BinaryOp(BinaryOp::Div, _, _) = rvalue {
                has_div = true;
            }
        }
    }
    assert!(has_div, "Division by zero should not be constant folded");
}
