use interpreter::value::Value;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use vm::object::{GcObj, MapObj, ArrayObj};
use vm::vm::VM;

fn json_to_viyal(vm: &mut VM, json: JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        },
        JsonValue::String(s) => Value::String(s),
        JsonValue::Array(arr) => {
            let elements = arr.into_iter().map(|item| json_to_viyal(vm, item)).collect();
            let handle = vm.allocate(GcObj::Array(ArrayObj { elements }));
            Value::Object(handle)
        },
        JsonValue::Object(obj) => {
            let mut entries = HashMap::new();
            for (k, v) in obj {
                entries.insert(k, json_to_viyal(vm, v));
            }
            let handle = vm.allocate(GcObj::Map(MapObj { entries }));
            Value::Object(handle)
        },
    }
}

fn viyal_to_json(vm: &VM, val: &Value) -> Result<JsonValue, String> {
    match val {
        Value::Null | Value::Void => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number(serde_json::Number::from(*i))),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Ok(JsonValue::Number(n))
            } else {
                Ok(JsonValue::Null)
            }
        },
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::Object(handle) => {
            match vm.get_obj(*handle) {
                Some(GcObj::Array(arr)) => {
                    let mut elements = Vec::new();
                    for el in &arr.elements {
                        elements.push(viyal_to_json(vm, el)?);
                    }
                    Ok(JsonValue::Array(elements))
                },
                Some(GcObj::Map(map_obj)) => {
                    let mut map = serde_json::Map::new();
                    for (k, v) in &map_obj.entries {
                        map.insert(k.clone(), viyal_to_json(vm, v)?);
                    }
                    Ok(JsonValue::Object(map))
                },
                _ => Err("Cannot serialize this object type to JSON".into()),
            }
        },
        _ => Err(format!("Cannot serialize type to JSON: {:?}", val)),
    }
}

pub fn parse(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("json.parse expects 1 argument (json_string)".into());
    }
    
    if let Value::String(json_str) = &args[0] {
        match serde_json::from_str::<JsonValue>(json_str) {
            Ok(json_val) => Ok(json_to_viyal(vm, json_val)),
            Err(e) => Err(format!("json.parse error: {}", e)),
        }
    } else {
        Err("json.parse argument must be a string".into())
    }
}

pub fn stringify(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("json.stringify expects 1 argument (value)".into());
    }
    
    let json_val = viyal_to_json(vm, &args[0])?;
    match serde_json::to_string(&json_val) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Err(format!("json.stringify error: {}", e)),
    }
}
