use interpreter::value::Value;
use vm::object::GcObj;

fn response_to_value(resp: ureq::Response) -> Result<Value, String> {
    resp.into_string().map(Value::String).map_err(|e| format!("http: failed to read response: {}", e))
}

pub fn get(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("http.get expects 1 argument (url)".into()); }
    if let Value::String(url) = &args[0] {
        ureq::get(url).call()
            .map_err(|e| format!("http.get failed: {}", e))
            .and_then(response_to_value)
    } else {
        Err("http.get url must be a string".into())
    }
}

pub fn post(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("http.post expects 2 arguments (url, body)".into()); }
    let url = match &args[0] { Value::String(s) => s, _ => return Err("http.post url must be a string".into()) };
    let body = match &args[1] { Value::String(s) => s.clone(), _ => return Err("http.post body must be a string".into()) };
    ureq::post(url).set("Content-Type", "text/plain").send_string(&body)
        .map_err(|e| format!("http.post failed: {}", e))
        .and_then(response_to_value)
}

pub fn post_json(vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("http.postJson expects 2 arguments (url, map_or_string)".into()); }
    let url = match &args[0] { Value::String(s) => s.clone(), _ => return Err("http.postJson url must be a string".into()) };
    // Accept either a raw JSON string or an Object (Map)
    let json_body = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Object(handle) => {
            // Serialize using serde_json
            match vm.get_obj(*handle) {
                Some(GcObj::Map(map_obj)) => {
                    let mut m = serde_json::Map::new();
                    for (k, v) in &map_obj.entries {
                        let jv = value_to_json(v)?;
                        m.insert(k.clone(), jv);
                    }
                    serde_json::to_string(&serde_json::Value::Object(m))
                        .map_err(|e| format!("http.postJson serialization error: {}", e))?
                },
                _ => return Err("http.postJson body must be a Map or JSON string".into()),
            }
        },
        _ => return Err("http.postJson body must be a Map or JSON string".into()),
    };
    ureq::post(&url).set("Content-Type", "application/json").send_string(&json_body)
        .map_err(|e| format!("http.postJson failed: {}", e))
        .and_then(response_to_value)
}

pub fn put(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 { return Err("http.put expects 2 arguments (url, body)".into()); }
    let url = match &args[0] { Value::String(s) => s, _ => return Err("http.put url must be a string".into()) };
    let body = match &args[1] { Value::String(s) => s.clone(), _ => return Err("http.put body must be a string".into()) };
    ureq::put(url).set("Content-Type", "text/plain").send_string(&body)
        .map_err(|e| format!("http.put failed: {}", e))
        .and_then(response_to_value)
}

pub fn delete(_vm: &mut crate::VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err("http.delete expects 1 argument (url)".into()); }
    if let Value::String(url) = &args[0] {
        ureq::delete(url).call()
            .map_err(|e| format!("http.delete failed: {}", e))
            .and_then(response_to_value)
    } else {
        Err("http.delete url must be a string".into())
    }
}

fn value_to_json(val: &Value) -> Result<serde_json::Value, String> {
    match val {
        Value::Null | Value::Void => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(*i))),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| "Cannot serialize NaN/Infinity to JSON".to_string()),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        _ => Err("Cannot serialize this type to JSON".into()),
    }
}

