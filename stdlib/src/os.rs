use interpreter::value::Value;
use vm::object::{GcObj, ArrayObj, MapObj};
use std::env;
use std::process::Command;
use std::collections::HashMap;

pub fn get_env(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("os.getenv expects 1 argument (key)".into());
    }
    if let Value::String(key) = &args[0] {
        match env::var(key) {
            Ok(val) => Ok(Value::String(val)),
            Err(_) => Ok(Value::Null),
        }
    } else {
        Err("os.getenv key must be a string".into())
    }
}

pub fn set_env(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("os.setenv expects 2 arguments (key, value)".into());
    }
    let key = match &args[0] { Value::String(k) => k, _ => return Err("os.setenv key must be a string".into()) };
    let val = match &args[1] { Value::String(v) => v, _ => return Err("os.setenv value must be a string".into()) };
    unsafe { env::set_var(key, val); }
    Ok(Value::Boolean(true))
}

pub fn env_all(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("os.envAll expects 0 arguments".into());
    }
    let mut entries = HashMap::new();
    for (key, val) in env::vars() {
        entries.insert(key, Value::String(val));
    }
    let handle = vm.allocate(GcObj::Map(MapObj { entries }));
    Ok(Value::Object(handle))
}

pub fn cwd(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("os.cwd expects 0 arguments".into());
    }
    env::current_dir()
        .map(|p| Value::String(p.to_string_lossy().to_string()))
        .map_err(|e| format!("os.cwd error: {}", e))
}

pub fn chdir(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("os.chdir expects 1 argument (path)".into());
    }
    if let Value::String(path) = &args[0] {
        env::set_current_dir(path)
            .map(|_| Value::Boolean(true))
            .map_err(|e| format!("os.chdir error: {}", e))
    } else {
        Err("os.chdir path must be a string".into())
    }
}

pub fn hostname(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("os.hostname expects 0 arguments".into());
    }
    // Use `hostname` command cross-platform
    let output = Command::new("hostname").output()
        .map_err(|e| format!("os.hostname error: {}", e))?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(Value::String(name))
}

pub fn platform(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("os.platform expects 0 arguments".into());
    }
    let name = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    Ok(Value::String(name.to_string()))
}

pub fn get_args(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("os.args expects 0 arguments".into());
    }
    let elements: Vec<Value> = env::args().map(Value::String).collect();
    let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
    Ok(Value::Object(handle))
}

pub fn sleep(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("os.sleep expects 1 argument (ms)".into());
    }
    let ms = match &args[0] {
        Value::Integer(n) => *n as u64,
        Value::Float(f) => *f as u64,
        _ => return Err("os.sleep ms must be a number".into()),
    };
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(Value::Void)
}

pub fn exit(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    let code = if args.is_empty() {
        0i32
    } else {
        match &args[0] {
            Value::Integer(n) => *n as i32,
            _ => return Err("os.exit code must be an integer".into()),
        }
    };
    std::process::exit(code);
}

pub fn execute(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("os.execute expects 1 argument (command)".into());
    }
    if let Value::String(cmd) = &args[0] {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", cmd]).output()
        } else {
            Command::new("sh").args(["-c", cmd]).output()
        };
        match output {
            Ok(out) => Ok(Value::String(String::from_utf8_lossy(&out.stdout).to_string())),
            Err(e) => Err(format!("os.execute error: {}", e)),
        }
    } else {
        Err("os.execute command must be a string".into())
    }
}

