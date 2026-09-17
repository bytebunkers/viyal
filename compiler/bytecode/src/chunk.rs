use crate::opcode::OpCode;
use interpreter::value::Value;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, opcode: OpCode, line: usize) {
        self.code.push(opcode);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn patch_jump(&mut self, offset: usize, target: usize) {
        match &mut self.code[offset] {
            OpCode::OpJump(val) | OpCode::OpJumpIfFalse(val) | OpCode::OpJumpIfOk(val) => *val = target,
            _ => panic!("Attempted to patch non-jump instruction"),
        }
    }
}
