use interpreter::value::Value;

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

pub fn math_abs(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.abs expects 1 argument".into()); }
    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err("math.abs expects a number".into()),
    }
}

pub fn math_sqrt(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.sqrt expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.sqrt())),
        None => Err("math.sqrt expects a number".into()),
    }
}

pub fn math_pow(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("math.pow expects 2 arguments".into()); }
    match (&args[0], &args[1]) {
        (Value::Integer(base), Value::Integer(exp)) if *exp >= 0 => Ok(Value::Integer(base.pow(*exp as u32))),
        _ => match (to_f64(&args[0]), to_f64(&args[1])) {
            (Some(b), Some(e)) => Ok(Value::Float(b.powf(e))),
            _ => Err("math.pow expects numbers".into()),
        }
    }
}

pub fn floor(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.floor expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Integer(f.floor() as i64)),
        None => Err("math.floor expects a number".into()),
    }
}

pub fn ceil(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.ceil expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Integer(f.ceil() as i64)),
        None => Err("math.ceil expects a number".into()),
    }
}

pub fn round(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.round expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Integer(f.round() as i64)),
        None => Err("math.round expects a number".into()),
    }
}

pub fn min(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("math.min expects 2 arguments".into()); }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer((*a).min(*b))),
        _ => match (to_f64(&args[0]), to_f64(&args[1])) {
            (Some(a), Some(b)) => Ok(Value::Float(a.min(b))),
            _ => Err("math.min expects numbers".into()),
        }
    }
}

pub fn max(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("math.max expects 2 arguments".into()); }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer((*a).max(*b))),
        _ => match (to_f64(&args[0]), to_f64(&args[1])) {
            (Some(a), Some(b)) => Ok(Value::Float(a.max(b))),
            _ => Err("math.max expects numbers".into()),
        }
    }
}

pub fn clamp(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 { return Err("math.clamp expects 3 arguments (val, min, max)".into()); }
    match (to_f64(&args[0]), to_f64(&args[1]), to_f64(&args[2])) {
        (Some(val), Some(lo), Some(hi)) => Ok(Value::Float(val.clamp(lo, hi))),
        _ => Err("math.clamp expects numbers".into()),
    }
}

pub fn log(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.log expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.ln())),
        None => Err("math.log expects a number".into()),
    }
}

pub fn log2(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.log2 expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.log2())),
        None => Err("math.log2 expects a number".into()),
    }
}

pub fn log10(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.log10 expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.log10())),
        None => Err("math.log10 expects a number".into()),
    }
}

pub fn sin(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.sin expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.sin())),
        None => Err("math.sin expects a number".into()),
    }
}

pub fn cos(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.cos expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.cos())),
        None => Err("math.cos expects a number".into()),
    }
}

pub fn tan(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("math.tan expects 1 argument".into()); }
    match to_f64(&args[0]) {
        Some(f) => Ok(Value::Float(f.tan())),
        None => Err("math.tan expects a number".into()),
    }
}

pub fn pi(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("math.PI expects 0 arguments".into()); }
    Ok(Value::Float(std::f64::consts::PI))
}

pub fn e(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("math.E expects 0 arguments".into()); }
    Ok(Value::Float(std::f64::consts::E))
}

pub fn random(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("math.random expects 0 arguments".into()); }
    // Simple LCG based random using system time as seed (no dep needed)
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() as u64;
    // xorshift64
    let mut x = seed.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x = x ^ (x >> 31);
    let f = (x >> 11) as f64 / (1u64 << 53) as f64;
    Ok(Value::Float(f))
}

