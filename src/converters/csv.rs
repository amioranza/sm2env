use crate::errors::SmError;
use serde_json::{Map, Value};

pub fn convert(data: &Map<String, Value>) -> Result<String, SmError> {
    let mut writer = csv::Writer::from_writer(vec![]);
    writer.write_record(["key", "value"])?;
    for (key, value) in data {
        let value_str = value
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| value.to_string());
        writer.write_record([key.as_str(), value_str.as_str()])?;
    }
    let inner = writer.into_inner().map_err(|e| SmError::FormatError(e.to_string()))?;
    let csv_content = String::from_utf8(inner)?;
    Ok(csv_content)
}
