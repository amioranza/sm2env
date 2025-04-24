#[cfg(test)]
mod tests {
    use crate::{process_json_secret, process_plain_text_secret, OutputFormat};
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_secret_to_env_format() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Test JSON with string values
        let json = serde_json::json!({
            "KEY1": "value1",
            "KEY2": "value2",
            "KEY3": "value with spaces"
        });

        // Process the JSON secret
        process_json_secret(json, &OutputFormat::Env, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();
        let expected = "KEY1=value1\nKEY2=value2\nKEY3=value with spaces\n";
        assert_eq!(content, expected);

        // Test JSON with quoted values
        let json = serde_json::json!({
            "KEY1": "\"quoted value\"",
            "KEY2": "\"another quoted value\""
        });

        // Process the JSON secret
        process_json_secret(json, &OutputFormat::Env, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();
        let expected = "KEY1=quoted value\nKEY2=another quoted value\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_plain_text_secret_to_env_format() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Test plain text with key=value format
        let text = "KEY1=value1\nKEY2=value2\nKEY3=value with spaces".to_string();
        process_plain_text_secret(text, &OutputFormat::Env, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();
        let expected = "KEY1=value1\nKEY2=value2\nKEY3=value with spaces\n";
        assert_eq!(content, expected);

        // Test plain text with quoted values
        let text = "KEY1=\"quoted value\"\nKEY2=\"another quoted value\"".to_string();
        process_plain_text_secret(text, &OutputFormat::Env, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();
        let expected = "KEY1=quoted value\nKEY2=another quoted value\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_json_secret_to_csv_format() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Test JSON with string values
        let json = serde_json::json!({
            "KEY1": "value1",
            "KEY2": "value2",
            "KEY3": "value with spaces"
        });

        // Process the JSON secret
        process_json_secret(json, &OutputFormat::Csv, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();

        // The CSV might have different line endings based on platform
        // We'll just check that it contains all expected keys and values
        assert!(content.contains("key,value"));
        assert!(content.contains("KEY1,value1"));
        assert!(content.contains("KEY2,value2"));
        assert!(content.contains("KEY3,value with spaces"));

        // Test JSON with quoted values
        let json = serde_json::json!({
            "KEY1": "\"quoted value\"",
            "KEY2": "\"another quoted value\""
        });

        // Process the JSON secret
        process_json_secret(json, &OutputFormat::Csv, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format - quotes should be properly handled
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("key,value"));
        assert!(content.contains("KEY1,quoted value"));
        assert!(content.contains("KEY2,another quoted value"));
    }

    #[test]
    fn test_plain_text_secret_to_csv_format() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Test plain text with key=value format
        let text = "KEY1=value1\nKEY2=value2\nKEY3=value with spaces".to_string();
        process_plain_text_secret(text, &OutputFormat::Csv, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();

        // Check CSV content
        assert!(content.contains("key,value"));
        assert!(content.contains("KEY1,value1"));
        assert!(content.contains("KEY2,value2"));
        assert!(content.contains("KEY3,value with spaces"));

        // Test plain text with quoted values
        let text = "KEY1=\"quoted value\"\nKEY2=\"another quoted value\"".to_string();
        process_plain_text_secret(text, &OutputFormat::Csv, &Some(file_path.clone())).unwrap();

        // Read the file and verify the format
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("key,value"));
        assert!(content.contains("KEY1,quoted value"));
        assert!(content.contains("KEY2,another quoted value"));
    }
}
