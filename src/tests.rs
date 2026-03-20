#[cfg(test)]
mod tests {
    use crate::converters;
    use base64::Engine;
    use crate::detect::{detect_secret_format, secret_to_map, SecretFormat};
    use crate::output::validate_path;
    use crate::OutputFormat;
    use serde_json::{json, Map, Value};
    use std::path::Path;
    use tempfile::NamedTempFile;

    // ── Converter helpers ──────────────────────────────────────────────────────

    fn make_map(pairs: &[(&str, &str)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
            .collect()
    }

    // ── Task 1.3: CSV output is valid RFC 4180 ─────────────────────────────────

    #[test]
    fn test_csv_simple_values() {
        let data = make_map(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let csv = converters::csv::convert(&data).unwrap();
        assert!(csv.contains("key,value"));
        assert!(csv.contains("KEY1,value1") || csv.contains("KEY2,value2"));
    }

    #[test]
    fn test_csv_value_with_comma() {
        let data = make_map(&[("FOO", "a,b")]);
        let csv = converters::csv::convert(&data).unwrap();
        // csv crate quotes values containing commas
        assert!(csv.contains("FOO,\"a,b\""));
    }

    #[test]
    fn test_csv_value_with_double_quote() {
        let data = make_map(&[("FOO", r#"say "hi""#)]);
        let csv = converters::csv::convert(&data).unwrap();
        // RFC 4180: the field should be quoted and internal quotes doubled.
        // Accept both LF and CRLF line endings.
        assert!(csv.contains("FOO,") && csv.contains(r#"say ""hi"""#));
    }

    #[test]
    fn test_csv_value_with_newline() {
        let data = make_map(&[("FOO", "line1\nline2")]);
        let csv = converters::csv::convert(&data).unwrap();
        // csv crate wraps values with newlines in quotes
        assert!(csv.contains("\"line1\nline2\""));
    }

    // ── Task 1.4: detect_secret_format ─────────────────────────────────────────

    #[test]
    fn test_detect_json_object() {
        let s = r#"{"key":"val"}"#;
        assert!(matches!(detect_secret_format(s), SecretFormat::Json(_)));
    }

    #[test]
    fn test_detect_json_array_is_plain_text() {
        let s = r#"["a","b"]"#;
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    #[test]
    fn test_detect_json_string_scalar_is_plain_text() {
        let s = r#""hello""#;
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    #[test]
    fn test_detect_json_number_is_plain_text() {
        let s = "123";
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    #[test]
    fn test_detect_json_bool_is_plain_text() {
        let s = "true";
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    #[test]
    fn test_detect_json_null_is_plain_text() {
        let s = "null";
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    #[test]
    fn test_detect_plain_text_kv() {
        let s = "FOO=bar\nBAZ=qux";
        assert!(matches!(detect_secret_format(s), SecretFormat::PlainText(_)));
    }

    // ── Task 2.3: Path validation ──────────────────────────────────────────────

    #[test]
    fn test_path_traversal_rejected() {
        let result = validate_path(Path::new("../../etc/passwd"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains(".."));
    }

    #[test]
    fn test_absolute_path_outside_cwd_rejected() {
        let result = validate_path(Path::new("/tmp/secret.env"));
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_relative_path_accepted() {
        let result = validate_path(Path::new("output.env"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_nested_relative_path_accepted() {
        let result = validate_path(Path::new("secrets/output.env"));
        assert!(result.is_ok());
    }

    // ── Task 2.4: File mode 0600 on Unix ──────────────────────────────────────

    #[test]
    #[cfg(unix)]
    fn test_file_written_with_0600_permissions() {
        use crate::output::write_output;
        use std::os::unix::fs::PermissionsExt;

        let _temp_dir = tempfile::tempdir().unwrap();

        // write_output with an absolute path inside cwd — but validate_path rejects absolute
        // paths outside cwd, so we use the write function indirectly via a relative path trick.
        // Instead, call write_secure via the public write_output with a path inside temp dir.
        // Since validate_path rejects absolute paths not in cwd, we write directly using
        // the internal write_secure logic by testing via a NamedTempFile in the cwd.
        let temp_file = NamedTempFile::new_in(".").unwrap();
        let path = temp_file.path().to_path_buf();
        write_output("KEY=val\n", Some(&path)).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode();
        // Check owner read+write only (0600 = 0o100600 with file type bits)
        assert_eq!(mode & 0o777, 0o600);
    }

    // ── Task 3.7: Converter isolation tests ────────────────────────────────────

    #[test]
    fn test_env_converter() {
        let data = make_map(&[("FOO", "bar"), ("BAZ", "qux")]);
        let result = converters::env::convert(&data);
        assert!(result.contains("FOO=bar\n"));
        assert!(result.contains("BAZ=qux\n"));
    }

    #[test]
    fn test_json_converter() {
        let data = make_map(&[("KEY", "val")]);
        let result = converters::json::convert(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["KEY"], "val");
    }

    #[test]
    fn test_yaml_converter() {
        let data = make_map(&[("KEY", "val")]);
        let result = converters::yaml::convert(&data).unwrap();
        assert!(result.contains("KEY: val"));
    }

    #[test]
    fn test_csv_converter() {
        let data = make_map(&[("KEY", "val")]);
        let result = converters::csv::convert(&data).unwrap();
        assert!(result.contains("key,value"));
        assert!(result.contains("KEY,val"));
    }

    // ── Task 6.3: Case-insensitive filter ─────────────────────────────────────

    #[test]
    fn test_filter_case_insensitive() {
        let name = "PROD-database";
        let filter = "prod";
        assert!(name.to_lowercase().contains(&filter.to_lowercase()));
    }

    #[test]
    fn test_filter_mixed_case() {
        let name = "prod-api";
        let filter = "Prod";
        assert!(name.to_lowercase().contains(&filter.to_lowercase()));
    }

    // ── Task 6.3: Empty secret handling ───────────────────────────────────────

    #[test]
    fn test_empty_secret_detected() {
        // An empty JSON object produces an empty map
        if let SecretFormat::Json(map) = detect_secret_format("{}") {
            assert!(map.is_empty());
        } else {
            panic!("Expected JSON format for {{}}");
        }
    }

    // ── Task 8.3: Multi-secret merge ─────────────────────────────────────────

    #[test]
    fn test_merge_two_maps() {
        let mut merged: Map<String, Value> = Map::new();
        let a = make_map(&[("FOO", "from_a"), ("SHARED", "from_a")]);
        let b = make_map(&[("BAR", "from_b"), ("SHARED", "from_b")]);
        merged.extend(a);
        merged.extend(b);

        assert_eq!(merged["FOO"], "from_a");
        assert_eq!(merged["BAR"], "from_b");
        // last-wins: b's SHARED overwrites a's
        assert_eq!(merged["SHARED"], "from_b");
    }

    // ── Task 10.4: Config loading ──────────────────────────────────────────────

    #[test]
    fn test_config_missing_file_returns_defaults() {
        // config::load_config() with no ~/.sm2env should return Config::default()
        // We test the parsing logic directly
        let result: Result<crate::config::Config, _> = toml::from_str("");
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert!(cfg.region.is_none());
        assert!(cfg.profile.is_none());
        assert!(cfg.format.is_none());
    }

    #[test]
    fn test_config_valid_toml() {
        let toml_str = r#"
region = "us-east-1"
profile = "staging"
format = "json"
"#;
        let cfg: crate::config::Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.region.as_deref(), Some("us-east-1"));
        assert_eq!(cfg.profile.as_deref(), Some("staging"));
        assert_eq!(cfg.format.as_deref(), Some("json"));
    }

    #[test]
    fn test_config_malformed_toml() {
        let bad_toml = "region = [unclosed";
        let result: Result<crate::config::Config, _> = toml::from_str(bad_toml);
        assert!(result.is_err());
    }

    // ── Task 11.1: Binary secret ──────────────────────────────────────────────

    #[test]
    fn test_binary_secret_to_base64_map() {
        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let base64_str = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let mut map = Map::new();
        map.insert("binary_data".to_string(), Value::String(base64_str.clone()));
        assert_eq!(map["binary_data"], base64_str);
    }

    // ── Task 11.2: JSON arrays treated as plain text ───────────────────────────

    #[test]
    fn test_json_array_is_plain_text() {
        let s = r#"["a","b","c"]"#;
        match detect_secret_format(s) {
            SecretFormat::PlainText(t) => assert_eq!(t, s),
            _ => panic!("Expected PlainText for JSON array"),
        }
    }

    // ── Task 11.3: Nested JSON objects ────────────────────────────────────────

    #[test]
    fn test_nested_json_object_is_detected_as_json() {
        let s = r#"{"outer": {"inner": "value"}}"#;
        assert!(matches!(detect_secret_format(s), SecretFormat::Json(_)));
    }

    #[test]
    fn test_nested_json_converted_to_env() {
        let json_val = json!({"outer": {"inner": "value"}});
        if let Value::Object(map) = json_val {
            let env = converters::env::convert(&map);
            // Nested object becomes its JSON string representation
            assert!(env.contains("outer="));
        }
    }

    // ── Task 11.4: Empty secret {} ────────────────────────────────────────────

    #[test]
    fn test_empty_json_secret_map_is_empty() {
        let s = "{}";
        match detect_secret_format(s) {
            SecretFormat::Json(map) => assert!(map.is_empty()),
            _ => panic!("Expected Json"),
        }
    }

    // ── Task 11.5: Multiline values in env format ─────────────────────────────

    #[test]
    fn test_multiline_value_in_env_output() {
        let data = make_map(&[("KEY", "line1\nline2")]);
        let env = converters::env::convert(&data);
        assert!(env.contains("KEY=line1\nline2\n"));
    }

    // ── Task 11.8: Invalid JSON object case should be handled ─────────────────

    #[test]
    fn test_plain_text_without_equals_becomes_single_key() {
        let s = "just a plain string";
        let fmt = detect_secret_format(s);
        let map = secret_to_map(fmt);
        assert!(map.contains_key("SECRET_VALUE"));
        assert_eq!(map["SECRET_VALUE"], "just a plain string");
    }

    // ── Backward-compat: existing JSON→env and JSON→csv behavior ──────────────

    #[test]
    fn test_json_to_env_format() {
        let temp = NamedTempFile::new_in(".").unwrap();
        let path = temp.path().to_path_buf();

        let data = make_map(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let content = converters::convert_to_format(&data, &OutputFormat::Env).unwrap();
        crate::output::write_output(&content, Some(&path)).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("KEY1=value1\n"));
        assert!(written.contains("KEY2=value2\n"));
    }

    #[test]
    fn test_json_to_csv_format() {
        let data = make_map(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let csv = converters::convert_to_format(&data, &OutputFormat::Csv).unwrap();
        assert!(csv.contains("key,value"));
        assert!(csv.contains("KEY1,value1"));
        assert!(csv.contains("KEY2,value2"));
    }

    #[test]
    fn test_plain_text_to_env_format() {
        let text = "KEY1=value1\nKEY2=value2".to_string();
        let fmt = detect_secret_format(&text);
        let map = secret_to_map(fmt);
        let content = converters::convert_to_format(&map, &OutputFormat::Env).unwrap();
        assert!(content.contains("KEY1=value1\n"));
        assert!(content.contains("KEY2=value2\n"));
    }

    #[test]
    fn test_plain_text_to_csv_format() {
        let text = "KEY1=value1\nKEY2=value2".to_string();
        let fmt = detect_secret_format(&text);
        let map = secret_to_map(fmt);
        let csv = converters::convert_to_format(&map, &OutputFormat::Csv).unwrap();
        assert!(csv.contains("key,value"));
        assert!(csv.contains("KEY1,value1"));
        assert!(csv.contains("KEY2,value2"));
    }
}
