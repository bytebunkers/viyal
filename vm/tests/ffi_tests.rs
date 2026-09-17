use vm::vm::{VM, InterpretResult};
use bytecode::chunk::Chunk;
use bytecode::opcode::OpCode;
use bytecode::compiler::CompiledProgram;
use interpreter::value::Value;
use vm::object::NativeBinding;

#[test]
fn test_ffi_call() {
    let mut chunk = Chunk::new();
    
    // Push arg
    let arg_idx = chunk.add_constant(Value::Integer(-42));
    chunk.write(OpCode::OpConstant(arg_idx), 1);
    
    // Push function name
    let name_idx = chunk.add_constant(Value::String("abs".to_string()));
    
    // Call Native
    chunk.write(OpCode::OpCallNative(name_idx, 1), 1);
    chunk.write(OpCode::OpPrint, 1);
    chunk.write(OpCode::OpReturn, 1);
    
    let program = CompiledProgram {
        main_chunk: chunk,
        method_chunks: vec![],
    };
    let mut vm = VM::new(program);
    
    // Register native function `abs`
    let abs_binding = NativeBinding::new("abs", |_vm, args| {
        if args.len() == 1 {
            if let Value::Integer(i) = args[0] {
                return Ok(Value::Integer(i.abs()));
            }
        }
        Err("Invalid arguments for abs".to_string())
    });
    
    vm.register_native(abs_binding);
    
    let mut output = String::new();
    let result = vm.run(&mut output);
    
    assert!(matches!(result, InterpretResult::Ok));
    assert_eq!(output, "42\n");
}
