use serde_json::{Map, Value};

pub fn convert(data: &Map<String, Value>) -> String {
    let mut content = String::new();
    for (key, value) in data {
        let value_str = value
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| value.to_string());
        content.push_str(&format!("{}={}\n", key, value_str.trim_matches('"')));
    }
    content
}
