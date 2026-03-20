use serde_json::{Map, Value};

pub enum SecretFormat {
    Json(Map<String, Value>),
    PlainText(String),
}

/// Detects the format of a secret string.
/// Only classifies as JSON if the top-level value is an Object.
/// Arrays, scalars, and null are treated as plain text.
pub fn detect_secret_format(secret: &str) -> SecretFormat {
    match serde_json::from_str::<Value>(secret) {
        Ok(Value::Object(obj)) => SecretFormat::Json(obj),
        _ => SecretFormat::PlainText(secret.to_string()),
    }
}

/// Parse key=value text into a Map, skipping blank lines and comments.
pub fn parse_env_vars(text: &str) -> Map<String, Value> {
    let mut map = Map::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim().trim_matches('"');
            if !key.is_empty() {
                map.insert(key.to_string(), Value::String(value.to_string()));
            }
        }
    }
    map
}

/// Convert a detected SecretFormat into a key-value Map.
pub fn secret_to_map(format: SecretFormat) -> Map<String, Value> {
    match format {
        SecretFormat::Json(map) => map,
        SecretFormat::PlainText(text) => {
            if text.contains('=') {
                parse_env_vars(&text)
            } else {
                let mut map = Map::new();
                map.insert("SECRET_VALUE".to_string(), Value::String(text));
                map
            }
        }
    }
}
