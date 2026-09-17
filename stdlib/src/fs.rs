use interpreter::value::Value;
use vm::object::{GcObj, ArrayObj, MapObj};
use std::fs;
use std::collections::HashMap;

pub fn read_text(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.readText expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        fs::read_to_string(path).map(Value::String).map_err(|e| format!("fs.readText error: {}", e))
    } else {
        Err("fs.readText path must be a string".into())
    }
}

pub fn write_text(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("fs.writeText expects 2 arguments (path, content)".into());
    }
    let path = match &args[0] { Value::String(p) => p, _ => return Err("fs.writeText path must be a string".into()) };
    let content = match &args[1] { Value::String(c) => c, _ => return Err("fs.writeText content must be a string".into()) };
    fs::write(path, content).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.writeText error: {}", e))
}

pub fn append(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("fs.append expects 2 arguments (path, content)".into());
    }
    let path = match &args[0] { Value::String(p) => p, _ => return Err("fs.append path must be a string".into()) };
    let content = match &args[1] { Value::String(c) => c, _ => return Err("fs.append content must be a string".into()) };
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).create(true).open(path)
        .map_err(|e| format!("fs.append error: {}", e))?;
    file.write_all(content.as_bytes()).map_err(|e| format!("fs.append write error: {}", e))?;
    Ok(Value::Boolean(true))
}

pub fn copy(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("fs.copy expects 2 arguments (src, dst)".into());
    }
    let src = match &args[0] { Value::String(p) => p, _ => return Err("fs.copy src must be a string".into()) };
    let dst = match &args[1] { Value::String(p) => p, _ => return Err("fs.copy dst must be a string".into()) };
    fs::copy(src, dst).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.copy error: {}", e))
}

pub fn delete(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.delete expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        let meta = fs::metadata(path).map_err(|e| format!("fs.delete error: {}", e))?;
        if meta.is_dir() {
            fs::remove_dir_all(path).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.delete error: {}", e))
        } else {
            fs::remove_file(path).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.delete error: {}", e))
        }
    } else {
        Err("fs.delete path must be a string".into())
    }
}

pub fn rename(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("fs.rename expects 2 arguments (src, dst)".into());
    }
    let src = match &args[0] { Value::String(p) => p, _ => return Err("fs.rename src must be a string".into()) };
    let dst = match &args[1] { Value::String(p) => p, _ => return Err("fs.rename dst must be a string".into()) };
    fs::rename(src, dst).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.rename error: {}", e))
}

pub fn mkdir(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.mkdir expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        fs::create_dir(path).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.mkdir error: {}", e))
    } else {
        Err("fs.mkdir path must be a string".into())
    }
}

pub fn mkdir_all(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.mkdirAll expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        fs::create_dir_all(path).map(|_| Value::Boolean(true)).map_err(|e| format!("fs.mkdirAll error: {}", e))
    } else {
        Err("fs.mkdirAll path must be a string".into())
    }
}

pub fn exists(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.exists expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        Ok(Value::Boolean(std::path::Path::new(path).exists()))
    } else {
        Err("fs.exists path must be a string".into())
    }
}

pub fn is_file(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.isFile expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        Ok(Value::Boolean(std::path::Path::new(path).is_file()))
    } else {
        Err("fs.isFile path must be a string".into())
    }
}

pub fn is_dir(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.isDir expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        Ok(Value::Boolean(std::path::Path::new(path).is_dir()))
    } else {
        Err("fs.isDir path must be a string".into())
    }
}

pub fn read_lines(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.readLines expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        let content = fs::read_to_string(path).map_err(|e| format!("fs.readLines error: {}", e))?;
        let elements: Vec<Value> = content.lines().map(|l| Value::String(l.to_string())).collect();
        let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
        Ok(Value::Object(handle))
    } else {
        Err("fs.readLines path must be a string".into())
    }
}

pub fn list_dir(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.listDir expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        let entries = fs::read_dir(path).map_err(|e| format!("fs.listDir error: {}", e))?;
        let mut elements = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("fs.listDir entry error: {}", e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            elements.push(Value::String(name));
        }
        let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
        Ok(Value::Object(handle))
    } else {
        Err("fs.listDir path must be a string".into())
    }
}

pub fn stat(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.stat expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        let meta = fs::metadata(path).map_err(|e| format!("fs.stat error: {}", e))?;
        let mut entries = HashMap::new();
        entries.insert("size".to_string(), Value::Integer(meta.len() as i64));
        entries.insert("isFile".to_string(), Value::Boolean(meta.is_file()));
        entries.insert("isDir".to_string(), Value::Boolean(meta.is_dir()));
        let modified = meta.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Value::Integer(d.as_secs() as i64))
            .unwrap_or(Value::Null);
        entries.insert("modified".to_string(), modified);
        let handle = vm.allocate(GcObj::Map(MapObj { entries }));
        Ok(Value::Object(handle))
    } else {
        Err("fs.stat path must be a string".into())
    }
}

