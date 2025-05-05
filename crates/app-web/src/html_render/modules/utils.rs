use {
    app_base::prelude::*,
    serde_json::Value,
    std::collections::HashMap,
    tera::{Result, helpers::tests::number_args_allowed}
};

pub fn dbg(args: &HashMap<String, Value>) -> Result<Value> {
    dbg!(args);
    Ok(Value::Null)
}

pub fn debug(args: &HashMap<String, Value>) -> Result<Value> {
    Ok(Value::String(format!("<pre>{args:#?}</pre>")))
}

pub fn is_debug(_args: &HashMap<String, Value>) -> Result<Value> {
    Ok(Value::from(Env::is_debug()))
}

pub fn is_null(value: Option<&Value>, params: &[Value]) -> Result<bool> {
    number_args_allowed("null", 0, params.len())?;

    match value {
        Some(&Value::Null) => Ok(true),
        _ => Ok(false)
    }
}
