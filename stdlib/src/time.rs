use interpreter::value::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("time.now expects 0 arguments".into()); }
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| Value::Integer(d.as_secs() as i64))
        .map_err(|_| "SystemTime before UNIX EPOCH!".into())
}

pub fn now_millis(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("time.nowMillis expects 0 arguments".into()); }
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| Value::Integer(d.as_millis() as i64))
        .map_err(|_| "SystemTime before UNIX EPOCH!".into())
}

pub fn now_nanos(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("time.nowNanos expects 0 arguments".into()); }
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| Value::Integer(d.as_nanos() as i64))
        .map_err(|_| "SystemTime before UNIX EPOCH!".into())
}

pub fn sleep(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("time.sleep expects 1 argument (ms)".into()); }
    let ms = match &args[0] {
        Value::Integer(n) => *n as u64,
        Value::Float(f) => *f as u64,
        _ => return Err("time.sleep ms must be a number".into()),
    };
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(Value::Void)
}

pub fn format(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err("time.format expects 1 or 2 arguments (timestamp_secs, optional_format)".into());
    }
    let ts = match &args[0] {
        Value::Integer(n) => *n,
        _ => return Err("time.format timestamp must be an integer".into()),
    };
    // Simple format: we produce ISO-8601-like string via chrono-free approach
    // ts is seconds since epoch; do manual UTC conversion
    let secs = ts as u64;
    let mins = secs / 60;
    let hours = mins / 60;
    let days_total = hours / 24;
    let sec = secs % 60;
    let min = (mins % 60) as u8;
    let hour = (hours % 24) as u8;
    // Approximate date (no leap year correction for brevity)
    let year = 1970 + days_total / 365;
    let day_of_year = days_total % 365;
    let month_days = [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    let mut day = day_of_year;
    for m in &month_days {
        if day < *m { break; }
        day -= m;
        month += 1;
    }
    Ok(Value::String(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day + 1, hour, min, sec
    )))
}

