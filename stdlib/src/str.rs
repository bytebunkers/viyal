use interpreter::value::Value;
use vm::object::{GcObj, ArrayObj};

pub fn str_len(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.len expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
        _ => Err("str.len argument must be a string".into()),
    }
}

pub fn str_trim(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.trim expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err("str.trim argument must be a string".into()),
    }
}

pub fn str_trim_left(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.trimLeft expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim_start().to_string())),
        _ => Err("str.trimLeft argument must be a string".into()),
    }
}

pub fn str_trim_right(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.trimRight expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim_end().to_string())),
        _ => Err("str.trimRight argument must be a string".into()),
    }
}

pub fn to_upper(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.toUpper expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err("str.toUpper argument must be a string".into()),
    }
}

pub fn to_lower(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.toLower expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err("str.toLower argument must be a string".into()),
    }
}

pub fn split(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.split expects 2 arguments (s, delim)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.split first arg must be string".into()) };
    let delim = match &args[1] { Value::String(d) => d, _ => return Err("str.split second arg must be string".into()) };
    let elements: Vec<Value> = s.split(delim.as_str()).map(|p| Value::String(p.to_string())).collect();
    let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
    Ok(Value::Object(handle))
}

pub fn join_str(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.join expects 2 arguments (arr, sep)".into()); }
    let sep = match &args[1] { Value::String(s) => s.clone(), _ => return Err("str.join sep must be a string".into()) };
    if let Value::Object(handle) = &args[0] {
        match vm.get_obj(*handle) {
            Some(vm::object::GcObj::Array(arr)) => {
                let parts: Vec<String> = arr.elements.iter().map(|v| {
                    if let Value::String(s) = v { s.clone() } else { format!("{}", v) }
                }).collect();
                Ok(Value::String(parts.join(&sep)))
            },
            _ => Err("str.join first argument must be an Array".into()),
        }
    } else {
        Err("str.join first argument must be an Array".into())
    }
}

pub fn contains(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.contains expects 2 arguments (s, sub)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.contains first arg must be string".into()) };
    let sub = match &args[1] { Value::String(d) => d, _ => return Err("str.contains second arg must be string".into()) };
    Ok(Value::Boolean(s.contains(sub.as_str())))
}

pub fn starts_with(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.startsWith expects 2 arguments (s, prefix)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.startsWith first arg must be string".into()) };
    let pre = match &args[1] { Value::String(p) => p, _ => return Err("str.startsWith second arg must be string".into()) };
    Ok(Value::Boolean(s.starts_with(pre.as_str())))
}

pub fn ends_with(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.endsWith expects 2 arguments (s, suffix)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.endsWith first arg must be string".into()) };
    let suf = match &args[1] { Value::String(p) => p, _ => return Err("str.endsWith second arg must be string".into()) };
    Ok(Value::Boolean(s.ends_with(suf.as_str())))
}

pub fn replace(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 { return Err("str.replace expects 3 arguments (s, from, to)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.replace first arg must be string".into()) };
    let from = match &args[1] { Value::String(f) => f, _ => return Err("str.replace second arg must be string".into()) };
    let to = match &args[2] { Value::String(t) => t, _ => return Err("str.replace third arg must be string".into()) };
    Ok(Value::String(s.replace(from.as_str(), to.as_str())))
}

pub fn repeat(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.repeat expects 2 arguments (s, n)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.repeat first arg must be string".into()) };
    let n = match &args[1] { Value::Integer(n) => *n as usize, _ => return Err("str.repeat second arg must be int".into()) };
    Ok(Value::String(s.repeat(n)))
}

pub fn index_of(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("str.indexOf expects 2 arguments (s, sub)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.indexOf first arg must be string".into()) };
    let sub = match &args[1] { Value::String(d) => d, _ => return Err("str.indexOf second arg must be string".into()) };
    match s.find(sub.as_str()) {
        Some(i) => Ok(Value::Integer(i as i64)),
        None => Ok(Value::Integer(-1)),
    }
}

pub fn slice(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 { return Err("str.slice expects 3 arguments (s, start, end)".into()); }
    let s = match &args[0] { Value::String(s) => s, _ => return Err("str.slice first arg must be string".into()) };
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = match &args[1] { Value::Integer(n) => *n, _ => return Err("str.slice start must be int".into()) };
    let end   = match &args[2] { Value::Integer(n) => *n, _ => return Err("str.slice end must be int".into()) };
    let start = (if start < 0 { (len + start).max(0) } else { start.min(len) }) as usize;
    let end   = (if end   < 0 { (len + end  ).max(0) } else { end  .min(len) }) as usize;
    let sliced: String = chars[start..end.max(start)].iter().collect();
    Ok(Value::String(sliced))
}

pub fn parse_int(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.parseInt expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => s.trim().parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| format!("str.parseInt cannot parse '{}' as int", s)),
        Value::Integer(n) => Ok(Value::Integer(*n)),
        _ => Err("str.parseInt argument must be a string".into()),
    }
}

pub fn parse_float(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.parseFloat expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => s.trim().parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("str.parseFloat cannot parse '{}' as float", s)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Integer(n) => Ok(Value::Float(*n as f64)),
        _ => Err("str.parseFloat argument must be a string".into()),
    }
}

pub fn chars(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("str.chars expects 1 argument".into()); }
    match &args[0] {
        Value::String(s) => {
            let elements: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
            Ok(Value::Object(handle))
        },
        _ => Err("str.chars argument must be a string".into()),
    }
}
