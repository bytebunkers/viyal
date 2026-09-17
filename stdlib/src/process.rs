use interpreter::value::Value;
use vm::object::{GcObj, MapObj};
use std::process::Command;
use std::collections::HashMap;

pub fn output(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err("process.output expects 1 or 2 arguments (cmd, optional_args_array)".into());
    }
    let cmd = match &args[0] { Value::String(s) => s.clone(), _ => return Err("process.output cmd must be a string".into()) };
    
    let out = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &cmd]).output()
    } else {
        Command::new("sh").args(["-c", &cmd]).output()
    }.map_err(|e| format!("process.output error: {}", e))?;
    
    let mut entries = HashMap::new();
    entries.insert("stdout".to_string(), Value::String(String::from_utf8_lossy(&out.stdout).to_string()));
    entries.insert("stderr".to_string(), Value::String(String::from_utf8_lossy(&out.stderr).to_string()));
    entries.insert("code".to_string(), Value::Integer(out.status.code().unwrap_or(-1) as i64));
    entries.insert("ok".to_string(), Value::Boolean(out.status.success()));
    
    let handle = vm.allocate(GcObj::Map(MapObj { entries }));
    Ok(Value::Object(handle))
}

pub fn pid(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("process.pid expects 0 arguments".into());
    }
    Ok(Value::Integer(std::process::id() as i64))
}

pub fn spawn(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("process.spawn expects 1 argument (cmd)".into());
    }
    let cmd = match &args[0] { Value::String(s) => s.clone(), _ => return Err("process.spawn cmd must be a string".into()) };
    
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &cmd]).spawn()
    } else {
        Command::new("sh").args(["-c", &cmd]).spawn()
    }.map_err(|e| format!("process.spawn error: {}", e))?;
    
    Ok(Value::Boolean(true))
}
