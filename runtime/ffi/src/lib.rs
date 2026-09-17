use interpreter::value::Value;

pub type NativeFunction = fn(&[Value]) -> Result<Value, String>;

#[derive(Clone)]
pub struct NativeBinding {
    pub name: String,
    pub func: NativeFunction,
}

impl NativeBinding {
    pub fn new(name: &str, func: NativeFunction) -> Self {
        Self {
            name: name.to_string(),
            func,
        }
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, String> {
        (self.func)(args)
    }
}
