use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Object(usize), // Points to GC Heap Handle
    Void,
    Null,
    Error(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Object(h) => write!(f, "<obj:{}>", h),
            Value::Void => write!(f, "void"),
            Value::Null => write!(f, "null"),
            Value::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}
