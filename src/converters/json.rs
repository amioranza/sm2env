use crate::errors::SmError;
use serde_json::{Map, Value};

pub fn convert(data: &Map<String, Value>) -> Result<String, SmError> {
    let obj = Value::Object(data.clone());
    Ok(serde_json::to_string_pretty(&obj)?)
}
