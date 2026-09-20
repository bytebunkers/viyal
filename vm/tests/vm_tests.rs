use bytecode::compiler::BytecodeCompiler;
use vm::vm::{VM, InterpretResult};
use ast::*;

#[test]
fn test_vm_execution() {
    let mut compiler = BytecodeCompiler::new();
    
    // expression: print(10 + 20 * 2);
    let left = Spanned { node: Expr::Literal(Literal::Integer(10)), span: 0..0 };
    let right_left = Spanned { node: Expr::Literal(Literal::Integer(20)), span: 0..0 };
    let right_right = Spanned { node: Expr::Literal(Literal::Integer(2)), span: 0..0 };
    
    let right = Spanned { node: Expr::Binary(Box::new(right_left), BinaryOp::Mul, Box::new(right_right)), span: 0..0 };
    let add = Spanned { node: Expr::Binary(Box::new(left), BinaryOp::Add, Box::new(right)), span: 0..0 };
    
    let target = Spanned { node: Expr::Identifier("print".to_string()), span: 0..0 };
    let call = Spanned { node: Expr::Call(Box::new(target), vec![], vec![add]), span: 0..0 };

    
    compiler.compile_expr(&call).unwrap();
    let program = compiler.compile_finish();
    
    let mut output = String::new();
    let mut vm = VM::new(program);
    let result = vm.run(&mut output);
    
    assert!(matches!(result, InterpretResult::Ok));
    assert_eq!(output, "50\n");
}
