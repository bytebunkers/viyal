use interpreter::value::Value;

pub fn print(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(Value::Void)
}

pub fn read_line(_vm: &mut crate::VM, _args: &[Value]) -> Result<Value, String> {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => Ok(Value::String(input.trim().to_string())),
        Err(e) => Err(format!("Failed to read line: {}", e)),
    }
}
