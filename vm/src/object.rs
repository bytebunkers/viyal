use std::collections::HashMap;
use bytecode::chunk::Chunk;
use interpreter::value::Value;

pub enum GcObj {
    String(String),
    Function(FunctionObj),
    Class(ClassObj),
    Instance(InstanceObj),
    Array(ArrayObj),
    Map(MapObj),
    NativeFunction(NativeFunctionObj),
    Module(ModuleObj),
}

pub struct ArrayObj {
    pub elements: Vec<Value>,
}

pub struct MapObj {
    pub entries: HashMap<String, Value>,
}

pub type NativeFunction = fn(&mut crate::vm::VM, &[Value]) -> Result<Value, String>;

#[derive(Clone)]
pub struct NativeBinding {
    pub name: String,
    pub func: NativeFunction,
}

impl NativeBinding {
    pub fn new(name: &str, func: NativeFunction) -> Self {
        Self { name: name.to_string(), func }
    }
    
    pub fn call(&self, vm: &mut crate::vm::VM, args: &[Value]) -> Result<Value, String> {
        (self.func)(vm, args)
    }
}

#[derive(Clone)]
pub struct NativeFunctionObj {
    pub name: String,
    pub arity: u8,
    pub func: NativeFunction,
}

pub struct ModuleObj {
    pub name: String,
    pub methods: HashMap<String, usize>, // maps method name to GcHandle of NativeFunctionObj
}

pub struct FunctionObj {
    pub name: String,
    pub arity: u8,
    pub chunk: Chunk,
}

pub struct ClassObj {
    pub name: String,
    pub methods: HashMap<String, usize>, // maps method name to GcHandle of FunctionObj
}

pub struct InstanceObj {
    pub class_handle: usize, // GcHandle of ClassObj
    pub fields: HashMap<String, Value>,
}
