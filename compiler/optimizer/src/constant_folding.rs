use ast::{BinaryOp, Literal};
use mir::ir::{BasicBlock, MirFunction, MirProgram, Operand, Rvalue, Statement};

pub fn fold_program(mut program: MirProgram) -> MirProgram {
    for func in program.functions.values_mut() {
        fold_function(func);
    }
    program
}

pub fn fold_function(func: &mut MirFunction) {
    for block in &mut func.basic_blocks {
        fold_block(block);
    }
}

fn fold_block(block: &mut BasicBlock) {
    for stmt in &mut block.statements {
        if let Statement::Assign(_, rvalue) = stmt {
            if let Rvalue::BinaryOp(op, Operand::Constant(left_lit), Operand::Constant(right_lit)) =
                rvalue
            {
                if let Some(result) = evaluate_binary_op(op, left_lit, right_lit) {
                    *rvalue = Rvalue::Use(Operand::Constant(result));
                }
            }
        }
    }
}

fn evaluate_binary_op(op: &BinaryOp, left: &Literal, right: &Literal) -> Option<Literal> {
    match (op, left, right) {
        (BinaryOp::Add, Literal::Integer(l), Literal::Integer(r)) => {
            l.checked_add(*r).map(Literal::Integer)
        }
        (BinaryOp::Sub, Literal::Integer(l), Literal::Integer(r)) => {
            l.checked_sub(*r).map(Literal::Integer)
        }
        (BinaryOp::Mul, Literal::Integer(l), Literal::Integer(r)) => {
            l.checked_mul(*r).map(Literal::Integer)
        }
        (BinaryOp::Div, Literal::Integer(l), Literal::Integer(r)) => {
            l.checked_div(*r).map(Literal::Integer)
        }

        // Float math
        (BinaryOp::Add, Literal::Float(l), Literal::Float(r)) => Some(Literal::Float(l + r)),
        (BinaryOp::Sub, Literal::Float(l), Literal::Float(r)) => Some(Literal::Float(l - r)),
        (BinaryOp::Mul, Literal::Float(l), Literal::Float(r)) => Some(Literal::Float(l * r)),
        (BinaryOp::Div, Literal::Float(l), Literal::Float(r)) => {
            if *r != 0.0 {
                Some(Literal::Float(l / r))
            } else {
                None
            }
        }

        // Comparisons
        (BinaryOp::Eq, Literal::Integer(l), Literal::Integer(r)) => Some(Literal::Boolean(l == r)),
        (BinaryOp::NotEq, Literal::Integer(l), Literal::Integer(r)) => {
            Some(Literal::Boolean(l != r))
        }
        (BinaryOp::Less, Literal::Integer(l), Literal::Integer(r)) => Some(Literal::Boolean(l < r)),
        (BinaryOp::Greater, Literal::Integer(l), Literal::Integer(r)) => {
            Some(Literal::Boolean(l > r))
        }
        (BinaryOp::LessEq, Literal::Integer(l), Literal::Integer(r)) => {
            Some(Literal::Boolean(l <= r))
        }
        (BinaryOp::GreaterEq, Literal::Integer(l), Literal::Integer(r)) => {
            Some(Literal::Boolean(l >= r))
        }

        // Fallback
        _ => None,
    }
}
