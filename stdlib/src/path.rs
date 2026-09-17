use interpreter::value::Value;
use std::path::Path;

pub fn join(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("path.join expects at least 2 arguments".into());
    }
    let mut result = std::path::PathBuf::new();
    for arg in args {
        if let Value::String(s) = arg {
            result.push(s);
        } else {
            return Err("path.join all arguments must be strings".into());
        }
    }
    Ok(Value::String(result.to_string_lossy().to_string()))
}

pub fn basename(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path.basename expects 1 argument (path)".into());
    }
    if let Value::String(p) = &args[0] {
        let name = Path::new(p).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(Value::String(name))
    } else {
        Err("path.basename argument must be a string".into())
    }
}

pub fn dirname(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path.dirname expects 1 argument (path)".into());
    }
    if let Value::String(p) = &args[0] {
        let parent = Path::new(p).parent()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(Value::String(parent))
    } else {
        Err("path.dirname argument must be a string".into())
    }
}

pub fn extension(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path.extension expects 1 argument (path)".into());
    }
    if let Value::String(p) = &args[0] {
        let ext = Path::new(p).extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(Value::String(ext))
    } else {
        Err("path.extension argument must be a string".into())
    }
}

pub fn stem(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path.stem expects 1 argument (path)".into());
    }
    if let Value::String(p) = &args[0] {
        let stem = Path::new(p).file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(Value::String(stem))
    } else {
        Err("path.stem argument must be a string".into())
    }
}

pub fn is_absolute(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path.isAbsolute expects 1 argument (path)".into());
    }
    if let Value::String(p) = &args[0] {
        Ok(Value::Boolean(Path::new(p).is_absolute()))
    } else {
        Err("path.isAbsolute argument must be a string".into())
    }
}
