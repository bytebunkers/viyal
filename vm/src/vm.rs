use bytecode::chunk::Chunk;
use bytecode::opcode::OpCode;
use interpreter::value::Value;
use gc::allocator::{GcAllocator, GcHandle};
use crate::object::NativeBinding;
use std::collections::{HashMap, HashSet};
use crate::object::{GcObj, ClassObj, InstanceObj, FunctionObj, ArrayObj, MapObj, NativeFunctionObj, ModuleObj};

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError(String),
    Breakpoint,
}

pub struct CallFrame {
    pub function_handle: usize, // Optional. If 0, it's the top-level script chunk.
    pub chunk: Chunk,
    pub ip: usize,
    pub base_slot: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    gc: GcAllocator<GcObj>,
    natives: HashMap<String, NativeBinding>,
    breakpoints: HashSet<usize>,
    is_paused: bool,
    globals: HashMap<String, Value>, // Simple globals for classes etc
    method_chunks: Vec<Chunk>,
    functions: HashMap<String, FunctionObj>,
}

impl VM {
    pub fn new(program: bytecode::compiler::CompiledProgram) -> Self {
        Self {
            frames: vec![CallFrame {
                function_handle: 0,
                chunk: program.main_chunk,
                ip: 0,
                base_slot: 0,
            }],
            stack: Vec::new(),
            gc: GcAllocator::new(),
            natives: HashMap::new(),
            breakpoints: HashSet::new(),
            is_paused: false,
            globals: HashMap::new(),
            method_chunks: program.method_chunks,
            functions: HashMap::new(),
        }
    }

    pub fn register_native(&mut self, binding: NativeBinding) {
        self.natives.insert(binding.name.clone(), binding);
    }
    
    pub fn add_breakpoint(&mut self, ip: usize) {
        self.breakpoints.insert(ip);
    }
    
    pub fn register_module(&mut self, name: &str, funcs: Vec<(&str, u8, crate::object::NativeFunction)>) {
        let mut module_methods = HashMap::new();
        for (func_name, arity, func) in funcs {
            let native_obj = NativeFunctionObj {
                name: func_name.to_string(),
                arity,
                func,
            };
            let handle = self.gc.allocate(GcObj::NativeFunction(native_obj));
            module_methods.insert(func_name.to_string(), handle.0);
        }
        
        let module_obj = ModuleObj {
            name: name.to_string(),
            methods: module_methods,
        };
        let handle = self.gc.allocate(GcObj::Module(module_obj));
        self.globals.insert(name.to_string(), Value::Object(handle.0));
    }
    
    pub fn allocate(&mut self, obj: GcObj) -> usize {
        self.gc.allocate(obj).0
    }
    
    pub fn get_obj(&self, handle: usize) -> Option<&GcObj> {
        self.gc.get(GcHandle(handle))
    }
    
    pub fn get_obj_mut(&mut self, handle: usize) -> Option<&mut GcObj> {
        self.gc.get_mut(GcHandle(handle))
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
    
    pub fn ip(&self) -> usize {
        self.frames.last().map(|f| f.ip).unwrap_or(0)
    }

    pub fn run(&mut self, output: &mut dyn std::fmt::Write) -> InterpretResult {
        loop {
            let frame = match self.frames.last_mut() {
                Some(f) => f,
                None => return InterpretResult::Ok, // execution finished
            };
            
            if frame.ip >= frame.chunk.code.len() {
                self.frames.pop();
                if self.frames.is_empty() {
                    return InterpretResult::Ok;
                }
                continue;
            }
            
            if self.breakpoints.contains(&frame.ip) && !self.is_paused {
                self.is_paused = true;
                return InterpretResult::Breakpoint;
            }
            self.is_paused = false;
            
            let instruction = frame.chunk.code[frame.ip].clone();
            frame.ip += 1;
            
            match instruction {
                OpCode::OpReturn => {
                    let result = self.stack.pop().unwrap_or(Value::Void);
                    let frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        return InterpretResult::Ok;
                    }
                    self.stack.truncate(frame.base_slot);
                    self.stack.push(result);
                },
                OpCode::OpConstant(idx) => {
                    let frame = self.frames.last().unwrap();
                    let constant = frame.chunk.constants[idx].clone();
                    self.stack.push(constant);
                },
                OpCode::OpCallNative(name_idx, arg_count) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    if let Value::String(name) = name_val {
                        let mut args = Vec::new();
                        for _ in 0..arg_count {
                            args.push(self.stack.pop().unwrap());
                        }
                        args.reverse(); // Stack pops in reverse order

                        if let Some(native) = self.natives.get(name).cloned() {
                            match native.call(self, &args) {
                                Ok(res) => self.stack.push(res),
                                Err(_) => return InterpretResult::RuntimeError("Native error".into()),
                            }
                        } else {
                            return InterpretResult::RuntimeError("Native not found".into());
                        }
                    } else {
                        return InterpretResult::RuntimeError("Native name is not string".into());
                    }
                },
                OpCode::OpAdd => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a + b)),
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a + b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpSubtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a - b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpMultiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a * b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpDivide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Integer(a / b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                },
                OpCode::OpNotEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a != b));
                },
                OpCode::OpLess => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a < b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpGreater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a > b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpLessEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a <= b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpGreaterEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => self.stack.push(Value::Boolean(a >= b)),
                        _ => return InterpretResult::RuntimeError("Type mismatch".into()),
                    }
                },
                OpCode::OpJump(offset) => {
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip = offset;
                },
                OpCode::OpJumpIfFalse(offset) => {
                    let val = self.stack.last().unwrap();
                    let is_truthy = match val {
                        Value::Boolean(false) | Value::Null => false,
                        _ => true,
                    };
                    if !is_truthy {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip = offset;
                    }
                },
                OpCode::OpPrint => {
                    let val = self.stack.pop().unwrap();
                    writeln!(output, "{}", val).unwrap();
                    self.stack.push(Value::Null);
                },
                OpCode::OpClass(name_idx) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = frame.chunk.constants[name_idx].clone();
                    if let Value::String(name) = name_val {
                        let class_obj = ClassObj {
                            name: name.clone(),
                            methods: HashMap::new(),
                        };
                        let handle = self.gc.allocate(GcObj::Class(class_obj));
                        let val = Value::Object(handle.0);
                        self.stack.push(val.clone());
                        self.globals.insert(name, val); // save globally for instantiation
                    }
                },
                OpCode::OpMethod(name_idx, chunk_idx, arity) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = frame.chunk.constants[name_idx].clone();
                    let method_name = if let Value::String(n) = name_val { n } else { return InterpretResult::RuntimeError("Method name not a string".into()); };
                    
                    let chunk = self.method_chunks[chunk_idx].clone();
                    let func_obj = FunctionObj {
                        name: method_name.clone(),
                        arity,
                        chunk,
                    };
                    let method_handle = self.gc.allocate(GcObj::Function(func_obj));
                    
                    let class_val = self.stack.last().unwrap().clone(); // Keep class on stack
                    
                    if let Value::Object(class_handle) = class_val {
                        if let Some(GcObj::Class(class_obj)) = self.gc.get_mut(GcHandle(class_handle)) {
                            class_obj.methods.insert(method_name, method_handle.0);
                        }
                    }
                },
                OpCode::OpFunction(name_idx, chunk_idx, arity) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = frame.chunk.constants[name_idx].clone();
                    let func_name = if let Value::String(n) = name_val { n } else { return InterpretResult::RuntimeError("Function name not a string".into()); };
                    
                    let chunk = self.method_chunks[chunk_idx].clone();
                    let func_obj = FunctionObj {
                        name: func_name.clone(),
                        arity,
                        chunk,
                    };
                    self.functions.insert(func_name, func_obj);
                },
                OpCode::OpCall(name_idx, arg_count) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    if let Value::String(func_name) = name_val {
                        if let Some(func_obj) = self.functions.get(func_name) {
                            if arg_count != func_obj.arity {
                                return InterpretResult::RuntimeError(format!("Expected {} arguments, got {}", func_obj.arity, arg_count));
                            }
                            let new_frame = CallFrame {
                                function_handle: 0,
                                chunk: func_obj.chunk.clone(),
                                ip: 0,
                                base_slot: self.stack.len() - arg_count as usize,
                            };
                            self.frames.push(new_frame);
                        } else if let Some(native_func) = self.natives.get(func_name).cloned() {
                            let args = self.stack.split_off(self.stack.len() - arg_count as usize);
                            match native_func.call(self, &args) {
                                Ok(ret_val) => self.stack.push(ret_val),
                                Err(msg) => return InterpretResult::RuntimeError(msg),
                            }
                        } else {
                            return InterpretResult::RuntimeError(format!("Function {} not found", func_name));
                        }
                    } else {
                        return InterpretResult::RuntimeError("Function name not a string".into());
                    }
                },
                OpCode::OpConstruct(name_idx, _arg_count) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    if let Value::String(name) = name_val {
                        if let Some(Value::Object(class_handle)) = self.globals.get(name).cloned() {
                            let instance = InstanceObj {
                                class_handle,
                                fields: HashMap::new(),
                            };
                            let handle = self.gc.allocate(GcObj::Instance(instance));
                            self.stack.push(Value::Object(handle.0));
                            // MVP: constructors just allocate instance, skipping constructor func call
                        } else {
                            return InterpretResult::RuntimeError(format!("Undefined class {}", name));
                        }
                    }
                },
                OpCode::OpInvoke(name_idx, arg_count) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    let method_name = if let Value::String(n) = name_val { n.clone() } else { return InterpretResult::RuntimeError("Bad method name".into()); };
                    
                    // The receiver (instance or module) is on stack BEFORE args
                    let receiver_idx = self.stack.len() - arg_count as usize - 1;
                    let receiver = self.stack[receiver_idx].clone();
                    
                    if let Value::Object(obj_handle) = receiver {
                        let method_handle = match self.gc.get(GcHandle(obj_handle)) {
                            Some(GcObj::Instance(inst)) => {
                                if let Some(GcObj::Class(cls)) = self.gc.get(GcHandle(inst.class_handle)) {
                                    cls.methods.get(&method_name).cloned()
                                } else { None }
                            },
                            Some(GcObj::Module(module)) => module.methods.get(&method_name).cloned(),
                            Some(GcObj::Array(_)) => {
                                if method_name == "push" {
                                    if arg_count != 1 {
                                        return InterpretResult::RuntimeError("Array.push expects 1 argument".into());
                                    }
                                    let arg = self.stack.pop().unwrap();
                                    self.stack.pop(); // pop receiver
                                    if let Some(GcObj::Array(arr)) = self.gc.get_mut(GcHandle(obj_handle)) {
                                        arr.elements.push(arg);
                                    }
                                    self.stack.push(Value::Null);
                                    continue;
                                }
                                return InterpretResult::RuntimeError(format!("Method {} not found on Array", method_name));
                            },
                            _ => return InterpretResult::RuntimeError("Cannot invoke method on this object type".into()),
                        };
                        
                        enum MethodAction {
                            PushFrame(CallFrame),
                            CallNative(crate::object::NativeFunctionObj),
                        }
                        
                        let action = if let Some(m_handle) = method_handle {
                            match self.gc.get(GcHandle(m_handle)) {
                                Some(GcObj::Function(func)) => {
                                    if func.arity != arg_count {
                                        return InterpretResult::RuntimeError(format!("Method {} expects {} arguments but got {}", method_name, func.arity, arg_count));
                                    }
                                    let new_frame = CallFrame {
                                        function_handle: m_handle,
                                        chunk: func.chunk.clone(),
                                        ip: 0,
                                        base_slot: receiver_idx,
                                    };
                                    Ok(MethodAction::PushFrame(new_frame))
                                },
                                Some(GcObj::NativeFunction(native)) => {
                                    if native.arity != arg_count {
                                        return InterpretResult::RuntimeError(format!("Native method {} expects {} arguments but got {}", method_name, native.arity, arg_count));
                                    }
                                    Ok(MethodAction::CallNative(native.clone()))
                                },
                                _ => Err(InterpretResult::RuntimeError("Method is not a function".into())),
                            }
                        } else {
                            Err(InterpretResult::RuntimeError(format!("Method {} not found", method_name)))
                        };
                        
                        match action {
                            Ok(MethodAction::PushFrame(frame)) => self.frames.push(frame),
                            Ok(MethodAction::CallNative(native)) => {
                                let mut args = Vec::new();
                                // args are from stack top to down
                                for _ in 0..arg_count {
                                    args.push(self.stack.pop().unwrap());
                                }
                                args.reverse();
                                self.stack.pop(); // pop the receiver
                                match (native.func)(self, &args) {
                                    Ok(res) => self.stack.push(res),
                                    Err(e) => return InterpretResult::RuntimeError(e),
                                }
                            },
                            Err(err) => return err,
                        }
                    } else {
                        return InterpretResult::RuntimeError("Cannot invoke method on non-object".into());
                    }
                },
                OpCode::OpGetProperty(name_idx) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    let field_name = if let Value::String(n) = name_val { n } else { return InterpretResult::RuntimeError("Bad field name".into()); };
                    
                    let instance_val = self.stack.pop().unwrap();
                    if let Value::Object(handle) = instance_val {
                        if let Some(GcObj::Instance(inst)) = self.gc.get(GcHandle(handle)) {
                            let val = inst.fields.get(field_name).cloned().unwrap_or(Value::Null);
                            self.stack.push(val);
                        }
                    }
                },
                OpCode::OpSetProperty(name_idx) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    let field_name = if let Value::String(n) = name_val { n.clone() } else { return InterpretResult::RuntimeError("Bad field name".into()); };
                    
                    let val = self.stack.pop().unwrap();
                    let instance_val = self.stack.pop().unwrap();
                    
                    if let Value::Object(handle) = instance_val {
                        if let Some(GcObj::Instance(inst)) = self.gc.get_mut(GcHandle(handle)) {
                            inst.fields.insert(field_name, val.clone());
                            self.stack.push(val); // Assignment evaluates to the value
                        }
                    }
                },
                OpCode::OpGetLocal(idx) => {
                    let frame = self.frames.last().unwrap();
                    let val = self.stack[frame.base_slot + idx].clone();
                    self.stack.push(val);
                },
                OpCode::OpSetLocal(idx) => {
                    let frame = self.frames.last().unwrap();
                    let val = self.stack.last().unwrap().clone();
                    self.stack[frame.base_slot + idx] = val;
                },
                OpCode::OpGetGlobal(name_idx) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    let global_name = if let Value::String(n) = name_val { n } else { return InterpretResult::RuntimeError("Bad global name".into()); };
                    
                    if let Some(val) = self.globals.get(global_name) {
                        self.stack.push(val.clone());
                    } else {
                        return InterpretResult::RuntimeError(format!("Undefined global variable '{}'", global_name));
                    }
                },
                OpCode::OpSetGlobal(name_idx) => {
                    let frame = self.frames.last().unwrap();
                    let name_val = &frame.chunk.constants[name_idx];
                    let global_name = if let Value::String(n) = name_val { n.clone() } else { return InterpretResult::RuntimeError("Bad global name".into()); };
                    
                    let val = self.stack.last().unwrap().clone();
                    self.globals.insert(global_name, val);
                },
                OpCode::OpArray(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..count {
                        elements.push(self.stack.pop().unwrap());
                    }
                    elements.reverse();
                    let array_obj = ArrayObj { elements };
                    let handle = self.gc.allocate(GcObj::Array(array_obj));
                    self.stack.push(Value::Object(handle.0));
                },
                OpCode::OpMap(count) => {
                    let mut entries = HashMap::new();
                    let mut pairs = Vec::new();
                    for _ in 0..count {
                        let value = self.stack.pop().unwrap();
                        let key = self.stack.pop().unwrap();
                        pairs.push((key, value));
                    }
                    pairs.reverse();
                    for (k, v) in pairs {
                        if let Value::String(s) = k {
                            entries.insert(s, v);
                        } else {
                            return InterpretResult::RuntimeError("Map keys must be strings".into());
                        }
                    }
                    let map_obj = MapObj { entries };
                    let handle = self.gc.allocate(GcObj::Map(map_obj));
                    self.stack.push(Value::Object(handle.0));
                },
                OpCode::OpIndex => {
                    let index_val = self.stack.pop().unwrap();
                    let coll_val = self.stack.pop().unwrap();
                    
                    if let Value::Object(handle) = coll_val {
                        match self.gc.get(GcHandle(handle)) {
                            Some(GcObj::Array(arr)) => {
                                let index = if let Value::Integer(i) = index_val { i } else { return InterpretResult::RuntimeError("Array index must be an integer".into()); };
                                if index < 0 || index as usize >= arr.elements.len() {
                                    return InterpretResult::RuntimeError("Array index out of bounds".into());
                                }
                                self.stack.push(arr.elements[index as usize].clone());
                            },
                            Some(GcObj::Map(map_obj)) => {
                                let key = if let Value::String(s) = index_val { s } else { return InterpretResult::RuntimeError("Map key must be a string".into()); };
                                let val = map_obj.entries.get(&key).cloned().unwrap_or(Value::Null);
                                self.stack.push(val);
                            },
                            _ => return InterpretResult::RuntimeError("Cannot index non-array/map object".into()),
                        }
                    } else if let Value::String(s) = coll_val {
                        let index = if let Value::Integer(i) = index_val { i } else { return InterpretResult::RuntimeError("String index must be an integer".into()); };
                        if index < 0 { return InterpretResult::RuntimeError("String index out of bounds".into()); }
                        if let Some(c) = s.chars().nth(index as usize) {
                            self.stack.push(Value::String(c.to_string()));
                        } else {
                            return InterpretResult::RuntimeError("String index out of bounds".into());
                        }
                    } else {
                        return InterpretResult::RuntimeError("Cannot index non-array/map/string type".into());
                    }
                },
                OpCode::OpIndexSet => {
                    let value = self.stack.pop().unwrap();
                    let index_val = self.stack.pop().unwrap();
                    let coll_val = self.stack.pop().unwrap();
                    
                    if let Value::Object(handle) = coll_val {
                        match self.gc.get_mut(GcHandle(handle)) {
                            Some(GcObj::Array(arr)) => {
                                let index = if let Value::Integer(i) = index_val { i } else { return InterpretResult::RuntimeError("Array index must be an integer".into()); };
                                if index < 0 || index as usize >= arr.elements.len() {
                                    return InterpretResult::RuntimeError("Array index out of bounds".into());
                                }
                                arr.elements[index as usize] = value.clone();
                                self.stack.push(value);
                            },
                            Some(GcObj::Map(map_obj)) => {
                                let key = if let Value::String(s) = index_val { s } else { return InterpretResult::RuntimeError("Map key must be a string".into()); };
                                map_obj.entries.insert(key, value.clone());
                                self.stack.push(value);
                            },
                            _ => return InterpretResult::RuntimeError("Cannot index assign non-array/map object".into()),
                        }
                    } else {
                        return InterpretResult::RuntimeError("Cannot assign to index of non-array/map type".into());
                    }

                },
                OpCode::OpLength => {
                    let coll_val = self.stack.pop().unwrap();
                    match coll_val {
                        Value::String(s) => {
                            self.stack.push(Value::Integer(s.chars().count() as i64));
                        },
                        Value::Object(handle) => {
                            match self.gc.get(GcHandle(handle)) {
                                Some(GcObj::Array(arr)) => {
                                    self.stack.push(Value::Integer(arr.elements.len() as i64));
                                },
                                Some(GcObj::Map(map)) => {
                                    self.stack.push(Value::Integer(map.entries.len() as i64));
                                },
                                _ => return InterpretResult::RuntimeError("Object does not have a length".into()),
                            }
                        },
                        _ => return InterpretResult::RuntimeError("Value does not have a length".into()),
                    }
                },
                OpCode::OpDuplicate => {
                    let v = self.stack.last().cloned().unwrap_or(Value::Null);
                    self.stack.push(v);
                }
                OpCode::OpPop => {
                    self.stack.pop();
                },
                OpCode::OpError => {
                    let val = self.stack.pop().unwrap();
                    let msg = if let Value::String(s) = val { s } else { "Unknown Error".to_string() };
                    self.stack.push(Value::Error(msg));
                },
                OpCode::OpTry => {
                    let val = self.stack.last().unwrap();
                    match val {
                        Value::Error(_) => {
                            // Automatically return the error
                            let result = self.stack.pop().unwrap();
                            let frame = self.frames.pop().unwrap();
                            if self.frames.is_empty() {
                                return InterpretResult::Ok;
                            }
                            self.stack.truncate(frame.base_slot);
                            self.stack.push(result);
                        },
                        Value::Null => {
                            // Automatically return null for optional Try
                            let result = self.stack.pop().unwrap();
                            let frame = self.frames.pop().unwrap();
                            if self.frames.is_empty() {
                                return InterpretResult::Ok;
                            }
                            self.stack.truncate(frame.base_slot);
                            self.stack.push(result);
                        },
                        _ => {
                            // If Ok, just proceed (value is already on stack)
                        }
                    }
                },
                OpCode::OpJumpIfOk(offset) => {
                    let val = self.stack.last().unwrap();
                    match val {
                        Value::Error(_) | Value::Null => {
                            // Do nothing, let it fall through to the 'or' block
                        },
                        _ => {
                            // Jump over the 'or' block
                            let frame = self.frames.last_mut().unwrap();
                            frame.ip = offset;
                        }
                    }
                },
            }
        }
    }
}
