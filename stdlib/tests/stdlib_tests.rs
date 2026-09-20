use vm::vm::{VM, InterpretResult};
use bytecode::chunk::Chunk;
use bytecode::opcode::OpCode;
use interpreter::value::Value;
use stdlib::register_stdlib;

#[test]
fn test_stdlib_math_abs() {
    let mut chunk = Chunk::new();
    
    // push arg: -100
    let arg_idx = chunk.add_constant(Value::Integer(-100));
    chunk.write(OpCode::OpConstant(arg_idx), 1);
    
    // push function name
    let name_idx = chunk.add_constant(Value::String("Math_abs".to_string()));
    
    // Call Native
    chunk.write(OpCode::OpCallNative(name_idx, 1), 1);
    chunk.write(OpCode::OpReturn, 1);
    
    let program = bytecode::compiler::CompiledProgram {
        main_chunk: chunk,
        method_chunks: vec![],
    };
    let mut vm = VM::new(program);
    register_stdlib(&mut vm);
    
    let mut output = String::new();
    let result = vm.run(&mut output);
    
    assert!(matches!(result, InterpretResult::Ok));
    // Verify result manually or let the VM push to stack and check stack.
    // In our VM architecture, the result is left on the stack.
}
