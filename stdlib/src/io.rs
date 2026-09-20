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

pub fn assert(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if let Some(Value::Boolean(b)) = args.first() {
        if *b {
            return Ok(Value::Void);
        } else {
            return Err("Assertion failed".to_string());
        }
    }
    Err("assert() requires a boolean argument".to_string())
}

pub fn assert_eq(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() == 2 {
        let actual = &args[0];
        let expected = &args[1];
        if actual == expected {
            return Ok(Value::Void);
        } else {
            return Err(format!("Assertion failed: expected {:?}, got {:?}", expected, actual));
        }
    }
    Err("assertEq() requires exactly 2 arguments".to_string())
}
