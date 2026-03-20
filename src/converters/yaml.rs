use crate::errors::SmError;
use serde_json::{Map, Value};

pub fn convert(data: &Map<String, Value>) -> Result<String, SmError> {
    let obj = Value::Object(data.clone());
    Ok(serde_yml::to_string(&obj)?)
}
