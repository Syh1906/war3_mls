use mlua::prelude::*;
use serde_json::Value as Jv;

pub fn install_json_lib(lua: &Lua) -> LuaResult<()> {
    let json = lua.create_table()?;
    json.set("encode", lua.create_function(json_encode)?)?;
    json.set("decode", lua.create_function(json_decode)?)?;
    lua.globals().set("json", json)?;
    Ok(())
}

fn json_encode(lua: &Lua, value: LuaValue) -> LuaResult<LuaValue> {
    let mut buf = Vec::new();
    encode_value(&value, &mut buf);
    Ok(LuaValue::String(lua.create_string(&buf)?))
}

fn json_decode(lua: &Lua, value: LuaValue) -> LuaResult<LuaValue> {
    let s = match value {
        LuaValue::Nil => return Ok(LuaValue::Nil),
        LuaValue::String(ref s) => match s.to_str() {
            Ok(s) if s.is_empty() => return Ok(LuaValue::Nil),
            Ok(s) => s.to_owned(),
            Err(e) => return Err(LuaError::RuntimeError(format!("json.decode: {e}"))),
        },
        _ => return Err(LuaError::RuntimeError("json.decode expects a string".into())),
    };
    let v: Jv = serde_json::from_str(&s)
        .map_err(|e| LuaError::RuntimeError(format!("json.decode: {e}")))?;
    json_to_lua(lua, &v)
}

// ── encode: build JSON as raw bytes (matching cjson byte-pass-through) ──

fn encode_value(v: &LuaValue, buf: &mut Vec<u8>) {
    match v {
        LuaValue::Nil => buf.extend_from_slice(b"null"),
        LuaValue::Boolean(b) => {
            buf.extend_from_slice(if *b { b"true" } else { b"false" });
        }
        LuaValue::Integer(i) => {
            let s = i.to_string();
            buf.extend_from_slice(s.as_bytes());
        }
        LuaValue::Number(n) => encode_number(*n, buf),
        LuaValue::String(s) => encode_lua_string(s, buf),
        LuaValue::Table(t) => encode_table(t, buf),
        _ => buf.extend_from_slice(b"null"),
    }
}

fn encode_number(n: f64, buf: &mut Vec<u8>) {
    if n.is_nan() || n.is_infinite() {
        buf.extend_from_slice(b"null");
        return;
    }
    if n == n.floor() && n.abs() < 1e15 {
        let s = format!("{}", n as i64);
        buf.extend_from_slice(s.as_bytes());
    } else {
        let s = format!("{:.14e}", n);
        let trimmed = trim_float(&s);
        buf.extend_from_slice(trimmed.as_bytes());
    }
}

fn trim_float(s: &str) -> String {
    if let Some(dot) = s.find('.') {
        if let Some(e_pos) = s[dot..].find(|c: char| c == 'e' || c == 'E') {
            let mantissa = &s[..dot + e_pos];
            let exp = &s[dot + e_pos..];
            let trimmed = mantissa.trim_end_matches('0');
            let trimmed = if trimmed.ends_with('.') {
                &trimmed[..trimmed.len() - 1]
            } else {
                trimmed
            };
            if exp == "e0" || exp == "E0" {
                return trimmed.to_string();
            }
            return format!("{}{}", trimmed, exp);
        }
    }
    s.to_string()
}

/// Write a Lua string as a JSON string value, passing raw bytes through
/// (only escaping characters required by JSON spec). This matches cjson behavior.
fn encode_lua_string(s: &mlua::String, buf: &mut Vec<u8>) {
    let bytes = s.as_bytes();
    buf.push(b'"');
    for &b in bytes.iter() {
        match b {
            b'\\' => buf.extend_from_slice(b"\\\\"),
            b'"' => buf.extend_from_slice(b"\\\""),
            b'\n' => buf.extend_from_slice(b"\\n"),
            b'\r' => buf.extend_from_slice(b"\\r"),
            b'\t' => buf.extend_from_slice(b"\\t"),
            0x00..=0x1f => {
                let esc = format!("\\u{:04x}", b);
                buf.extend_from_slice(esc.as_bytes());
            }
            _ => buf.push(b),
        }
    }
    buf.push(b'"');
}

// cjson: empty table → object, sequential 1..n → array, otherwise → object
fn is_array_table(t: &LuaTable) -> bool {
    let mut n = 0i64;
    for pair in t.pairs::<LuaValue, LuaValue>() {
        if pair.is_ok() {
            n += 1;
        }
    }
    if n == 0 {
        return false;
    }
    for i in 1..=n {
        match t.raw_get::<LuaValue>(i) {
            Ok(LuaValue::Nil) | Err(_) => return false,
            _ => {}
        }
    }
    true
}

fn encode_table(t: &LuaTable, buf: &mut Vec<u8>) {
    if is_array_table(t) {
        buf.push(b'[');
        let len = t.raw_len();
        for i in 1..=len {
            if i > 1 {
                buf.push(b',');
            }
            let v: LuaValue = t.raw_get(i as i64).unwrap_or(LuaValue::Nil);
            encode_value(&v, buf);
        }
        buf.push(b']');
    } else {
        buf.push(b'{');
        let mut first = true;
        for pair in t.pairs::<LuaValue, LuaValue>() {
            if let Ok((k, v)) = pair {
                if !first {
                    buf.push(b',');
                }
                first = false;
                encode_key(&k, buf);
                buf.push(b':');
                encode_value(&v, buf);
            }
        }
        buf.push(b'}');
    }
}

fn encode_key(k: &LuaValue, buf: &mut Vec<u8>) {
    match k {
        LuaValue::String(s) => encode_lua_string(s, buf),
        LuaValue::Integer(i) => {
            let s = i.to_string();
            buf.push(b'"');
            buf.extend_from_slice(s.as_bytes());
            buf.push(b'"');
        }
        LuaValue::Number(n) if *n == n.floor() && n.abs() < 1e15 => {
            let s = format!("{}", *n as i64);
            buf.push(b'"');
            buf.extend_from_slice(s.as_bytes());
            buf.push(b'"');
        }
        LuaValue::Number(n) => {
            let s = format!("{n}");
            buf.push(b'"');
            buf.extend_from_slice(s.as_bytes());
            buf.push(b'"');
        }
        _ => buf.extend_from_slice(b"\"\""),
    }
}

// ── decode: still uses serde_json (input is always valid JSON/UTF-8) ──

fn json_to_lua(lua: &Lua, v: &Jv) -> LuaResult<LuaValue> {
    Ok(match v {
        Jv::Null => LuaValue::Nil,
        Jv::Bool(b) => LuaValue::Boolean(*b),
        Jv::Number(n) => {
            if let Some(i) = n.as_i64() {
                LuaValue::Integer(i)
            } else {
                LuaValue::Number(n.as_f64().unwrap_or(0.0))
            }
        }
        Jv::String(s) => LuaValue::String(lua.create_string(s)?),
        Jv::Array(arr) => {
            let t = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                t.raw_set(i as i64 + 1, json_to_lua(lua, v)?)?;
            }
            LuaValue::Table(t)
        }
        Jv::Object(map) => {
            let t = lua.create_table()?;
            for (k, v) in map {
                t.raw_set(lua.create_string(k.as_str())?, json_to_lua(lua, v)?)?;
            }
            LuaValue::Table(t)
        }
    })
}
