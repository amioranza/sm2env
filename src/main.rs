use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client;
use base64::Engine;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::fmt;
use std::fs;

mod tests;

#[derive(Parser)]
#[command(
    name = "sm2env",
    about = "A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.",
    version = "0.1.5",
    author = "Your Name",
    long_about = "sm2env is a command-line tool that helps retrieve secrets from AWS Secrets Manager \
                  and store them in a .env file for easy environment variable management."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch a specific secret and save it as a .env file
    Get {
        /// The name of the secret to retrieve
        secret_name: String,

        /// Specify the output format (stdout, json, env, yaml, csv)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Env)]
        output: OutputFormat,

        /// Specify a custom file path to write the output to
        #[arg(short, long)]
        file: Option<String>,
    },
    /// List all available secrets
    List {
        /// Filter secrets by a specific text
        #[arg(short, long)]
        filter: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Stdout,
    Json,
    Env,
    Yaml,
    Csv,
}

// Implement Display for OutputFormat
impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Stdout => write!(f, "stdout"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Env => write!(f, "env"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Use the latest behavior version for AWS SDK
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let client = Client::new(&config);

    match &cli.command {
        Some(Commands::Get {
            secret_name,
            output,
            file,
        }) => get_secret(&client, secret_name, output, file).await?,
        Some(Commands::List { filter }) => list_secrets(&client, filter.clone()).await?,
        None => {
            // If no arguments are provided, print the help message
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}

async fn get_secret(
    client: &Client,
    secret_name: &str,
    output_format: &OutputFormat,
    file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;

    // Handle different secret formats
    if let Some(secret_string) = response.secret_string {
        // Process the secret based on its detected format
        match detect_secret_format(&secret_string) {
            SecretFormat::Json(json_value) => {
                process_json_secret(json_value, output_format, file)?;
            }
            SecretFormat::PlainText(text) => {
                process_plain_text_secret(text, output_format, file)?;
            }
        }
    } else if let Some(secret_binary) = response.secret_binary {
        // Handle binary secrets
        process_binary_secret(secret_binary, output_format, file)?;
    } else {
        println!("No secret content found in the response.");
    }

    Ok(())
}

/// Detects the format of a secret string
enum SecretFormat {
    Json(serde_json::Value),
    PlainText(String),
}

fn detect_secret_format(secret: &str) -> SecretFormat {
    // Try to parse as JSON first
    match serde_json::from_str::<serde_json::Value>(secret) {
        Ok(json) => SecretFormat::Json(json),
        Err(_) => SecretFormat::PlainText(secret.to_string()),
    }
}

/// Process JSON formatted secrets
pub fn process_json_secret(
    json: serde_json::Value,
    output_format: &OutputFormat,
    file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // If file option is provided, write to the file regardless of output format
    if let Some(file_path) = file {
        match output_format {
            OutputFormat::Stdout => {
                // For stdout with file option, write the raw content to the file
                let mut content = String::new();
                if let Some(obj) = json.as_object() {
                    for (key, value) in obj {
                        let value_str = value
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| value.to_string());
                        content.push_str(&format!("{}={}\n", key, value_str.trim_matches('"')));
                    }
                    fs::write(file_path, content)?;
                    println!("Secret written to file: {}", file_path);
                } else {
                    // If not an object, write the JSON as is
                    let content = serde_json::to_string(&json)?;
                    fs::write(file_path, content)?;
                    println!("Secret written to file: {}", file_path);
                }
            }
            OutputFormat::Json => {
                let json_content = serde_json::to_string_pretty(&json)?;
                fs::write(file_path, json_content)?;
                println!("JSON file created successfully at {}", file_path);
            }
            OutputFormat::Env => {
                let mut env_content = String::new();
                if let Some(obj) = json.as_object() {
                    for (key, value) in obj {
                        let value_str = value
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| value.to_string());
                        env_content.push_str(&format!("{}={}\n", key, value_str.trim_matches('"')));
                    }
                    fs::write(file_path, env_content)?;
                    println!(".env file created successfully at {}", file_path);
                } else {
                    println!("The secret is not a valid JSON object.");
                }
            }
            OutputFormat::Yaml => {
                let yaml_content = serde_yaml::to_string(&json)?;
                fs::write(file_path, yaml_content)?;
                println!("YAML file created successfully at {}", file_path);
            }
            OutputFormat::Csv => {
                let csv_content = convert_json_to_csv(&json)?;
                fs::write(file_path, csv_content)?;
                println!("CSV file created successfully at {}", file_path);
            }
        }
        return Ok(());
    }

    // Original behavior when no file is specified
    match output_format {
        OutputFormat::Stdout => {
            // Display to stdout in .env format
            if let Some(obj) = json.as_object() {
                for (key, value) in obj {
                    let value_str = value
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| value.to_string());
                    println!("{}={}", key, value_str.trim_matches('"'));
                }
            } else {
                println!("The secret is not a valid JSON object.");
            }
        }
        OutputFormat::Json => {
            // Save as JSON file
            let json_content = serde_json::to_string_pretty(&json)?;
            fs::write("secret.json", json_content)?;
            println!("JSON file created successfully!");
        }
        OutputFormat::Env => {
            // Save as .env file
            let mut env_content = String::new();
            if let Some(obj) = json.as_object() {
                for (key, value) in obj {
                    let value_str = value
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| value.to_string());
                    env_content.push_str(&format!("{}={}\n", key, value_str.trim_matches('"')));
                }
                fs::write(".env", env_content)?;
                println!(".env file created successfully!");
            } else {
                println!("The secret is not a valid JSON object.");
            }
        }
        OutputFormat::Yaml => {
            // Save as YAML file
            let yaml_content = serde_yaml::to_string(&json)?;
            fs::write("secret.yaml", yaml_content)?;
            println!("YAML file created successfully!");
        }
        OutputFormat::Csv => {
            // Save as CSV file
            let csv_content = convert_json_to_csv(&json)?;
            fs::write("secret.csv", csv_content)?;
            println!("CSV file created successfully!");
        }
    }
    Ok(())
}

/// Process plain text secrets
pub fn process_plain_text_secret(
    text: String,
    output_format: &OutputFormat,
    file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // If file option is provided, write to the file regardless of output format
    if let Some(file_path) = file {
        match output_format {
            OutputFormat::Stdout => {
                // For stdout with file option, write the raw content to the file
                fs::write(file_path, &text)?;
                println!("Secret written to file: {}", file_path);
            }
            OutputFormat::Json => {
                // Check if the text is in env var format (key=value pairs)
                if text.contains('=') && text.contains('\n') {
                    // Convert env var format to JSON
                    let json_content = convert_env_to_json(&text)?;
                    fs::write(file_path, json_content)?;
                    println!("JSON file created successfully at {}", file_path);
                } else {
                    let json = serde_json::json!({ "value": text });
                    let json_content = serde_json::to_string_pretty(&json)?;
                    fs::write(file_path, json_content)?;
                    println!("JSON file created successfully at {}", file_path);
                }
            }
            OutputFormat::Env => {
                // If the text contains key=value pairs, process and write
                if text.contains('=') {
                    // Process the text to remove quotes from values
                    let processed_text = process_env_text(&text);
                    // Ensure there's a trailing newline
                    let text_with_newline = if processed_text.ends_with('\n') {
                        processed_text
                    } else {
                        format!("{}\n", processed_text)
                    };
                    fs::write(file_path, text_with_newline)?;
                } else {
                    // Use SECRET_VALUE as default key only if it's not a key=value pair
                    fs::write(file_path, format!("SECRET_VALUE={}\n", text))?;
                }
                println!(".env file created successfully at {}", file_path);
            }
            OutputFormat::Yaml => {
                // Check if the text is in env var format (key=value pairs)
                if text.contains('=') && text.contains('\n') {
                    // Convert env var format to YAML
                    let env_vars = parse_env_vars(&text);
                    let yaml_content = serde_yaml::to_string(&env_vars)?;
                    fs::write(file_path, yaml_content)?;
                    println!("YAML file created successfully at {}", file_path);
                } else {
                    let yaml_content =
                        serde_yaml::to_string(&serde_json::json!({ "value": text }))?;
                    fs::write(file_path, yaml_content)?;
                    println!("YAML file created successfully at {}", file_path);
                }
            }
            OutputFormat::Csv => {
                // Check if the text is in env var format (key=value pairs)
                if text.contains('=') && text.contains('\n') {
                    // Convert env var format to CSV
                    let csv_content = convert_env_to_csv(&text)?;
                    fs::write(file_path, csv_content)?;
                    println!("CSV file created successfully at {}", file_path);
                } else {
                    // Single value, save as a CSV with a default key
                    let csv_content = format!("key,value\nvalue,{}", escape_csv_field(&text));
                    fs::write(file_path, csv_content)?;
                    println!("CSV file created successfully at {}", file_path);
                }
            }
        }
        return Ok(());
    }

    // Original behavior when no file is specified
    match output_format {
        OutputFormat::Stdout => {
            // Display to stdout directly
            println!("{}", text);
        }
        OutputFormat::Json => {
            // Check if the text is in env var format (key=value pairs)
            if text.contains('=') && text.contains('\n') {
                // Convert env var format to JSON
                let json_content = convert_env_to_json(&text)?;
                fs::write("secret.json", json_content)?;
                println!("JSON file created successfully!");
            } else {
                // Save as JSON file with a default key
                let json = serde_json::json!({ "value": text });
                let json_content = serde_json::to_string_pretty(&json)?;
                fs::write("secret.json", json_content)?;
                println!("JSON file created successfully!");
            }
        }
        OutputFormat::Env => {
            // If the text contains key=value pairs, process and write
            if text.contains('=') {
                // Process the text to remove quotes from values
                let processed_text = process_env_text(&text);
                // Ensure there's a trailing newline
                let text_with_newline = if processed_text.ends_with('\n') {
                    processed_text
                } else {
                    format!("{}\n", processed_text)
                };
                fs::write(".env", text_with_newline)?;
            } else {
                // Use SECRET_VALUE as default key only if it's not a key=value pair
                fs::write(".env", format!("SECRET_VALUE={}\n", text))?;
            }
            println!(".env file created successfully!");
        }
        OutputFormat::Yaml => {
            // Check if the text is in env var format (key=value pairs)
            if text.contains('=') && text.contains('\n') {
                // Convert env var format to YAML
                let env_vars = parse_env_vars(&text);
                let yaml_content = serde_yaml::to_string(&env_vars)?;
                fs::write("secret.yaml", yaml_content)?;
                println!("YAML file created successfully!");
            } else {
                // Save as YAML file with a default key
                let yaml_content = serde_yaml::to_string(&serde_json::json!({ "value": text }))?;
                fs::write("secret.yaml", yaml_content)?;
                println!("YAML file created successfully!");
            }
        }
        OutputFormat::Csv => {
            // Check if the text is in env var format (key=value pairs)
            if text.contains('=') && text.contains('\n') {
                // Convert env var format to CSV
                let csv_content = convert_env_to_csv(&text)?;
                fs::write("secret.csv", csv_content)?;
                println!("CSV file created successfully!");
            } else {
                // Single value, save as a CSV with a default key
                let csv_content = format!("key,value\nvalue,{}", escape_csv_field(&text));
                fs::write("secret.csv", csv_content)?;
                println!("CSV file created successfully!");
            }
        }
    }
    Ok(())
}

/// Process environment text to remove quotes from values
fn process_env_text(text: &str) -> String {
    let mut result = String::new();

    for line in text.lines() {
        if let Some(pos) = line.find('=') {
            let key = &line[..pos];
            let value = &line[pos + 1..];

            // Remove quotes from value
            let processed_value = value.trim_matches('"');
            result.push_str(&format!("{}={}\n", key, processed_value));
        } else {
            // If no equals sign, keep the line as is
            result.push_str(line);
            result.push('\n');
        }
    }

    // Remove trailing newline if we added one
    if result.ends_with('\n') && !text.ends_with('\n') {
        result.pop();
    }

    result
}

/// Parse environment variables from text, removing blank and commented lines
fn parse_env_vars(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut env_vars = serde_json::Map::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip blank lines and commented lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split by the first equals sign
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();

            // Add to our map if key is not empty
            if !key.is_empty() {
                env_vars.insert(
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }
    }

    env_vars
}

/// Convert environment variables format to JSON
fn convert_env_to_json(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let env_vars = parse_env_vars(text);
    let json_value = serde_json::Value::Object(env_vars);
    Ok(serde_json::to_string_pretty(&json_value)?)
}

/// Process binary secrets
fn process_binary_secret(
    binary: aws_sdk_secretsmanager::primitives::Blob,
    output_format: &OutputFormat,
    file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let binary_data = binary.as_ref();

    // If file option is provided, write to the file regardless of output format
    if let Some(file_path) = file {
        match output_format {
            OutputFormat::Stdout => {
                // For binary data with file option, write the raw binary data to the file
                fs::write(file_path, binary_data)?;
                println!(
                    "Binary secret ({} bytes) written to file: {}",
                    binary_data.len(),
                    file_path
                );
            }
            OutputFormat::Json => {
                let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
                let json = serde_json::json!({ "binary_data": base64_str });
                let json_content = serde_json::to_string_pretty(&json)?;
                fs::write(file_path, json_content)?;
                println!(
                    "JSON file with base64-encoded binary data created successfully at {}",
                    file_path
                );
            }
            OutputFormat::Env => {
                let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
                fs::write(file_path, format!("BINARY_SECRET={}\n", base64_str))?;
                println!(
                    ".env file with base64-encoded binary data created successfully at {}",
                    file_path
                );
            }
            OutputFormat::Yaml => {
                let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
                let yaml_content =
                    serde_yaml::to_string(&serde_json::json!({ "binary_data": base64_str }))?;
                fs::write(file_path, yaml_content)?;
                println!(
                    "YAML file with base64-encoded binary data created successfully at {}",
                    file_path
                );
            }
            OutputFormat::Csv => {
                let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
                let csv_content =
                    format!("key,value\nbinary_data,{}", escape_csv_field(&base64_str));
                fs::write(file_path, csv_content)?;
                println!(
                    "CSV file with base64-encoded binary data created successfully at {}",
                    file_path
                );
            }
        }
        return Ok(());
    }

    // Original behavior when no file is specified
    match output_format {
        OutputFormat::Stdout => {
            // For binary data, just indicate it's binary and its size
            println!("Binary secret data ({} bytes)", binary_data.len());
        }
        OutputFormat::Json => {
            // Save binary data as base64 in a JSON file
            let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
            let json = serde_json::json!({ "binary_data": base64_str });
            let json_content = serde_json::to_string_pretty(&json)?;
            fs::write("secret.json", json_content)?;
            println!("JSON file with base64-encoded binary data created successfully!");
        }
        OutputFormat::Env => {
            // Save binary data as base64 in .env file
            let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
            fs::write(".env", format!("BINARY_SECRET={}\n", base64_str))?;
            println!(".env file with base64-encoded binary data created successfully!");
        }
        OutputFormat::Yaml => {
            // Save binary data as base64 in a YAML file
            let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
            let yaml_content =
                serde_yaml::to_string(&serde_json::json!({ "binary_data": base64_str }))?;
            fs::write("secret.yaml", yaml_content)?;
            println!("YAML file with base64-encoded binary data created successfully!");
        }
        OutputFormat::Csv => {
            // Save binary data as base64 in a CSV file
            let base64_str = base64::engine::general_purpose::STANDARD.encode(binary_data);
            let csv_content = format!("key,value\nbinary_data,{}", escape_csv_field(&base64_str));
            fs::write("secret.csv", csv_content)?;
            println!("CSV file with base64-encoded binary data created successfully!");
        }
    }
    Ok(())
}

/// Convert JSON to CSV format
fn convert_json_to_csv(json: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    let mut writer = csv::Writer::from_writer(vec![]);

    // Write the header row
    writer.write_record(&["key", "value"])?;

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            let value_str = value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| value.to_string());

            // Remove quotes from value if it's a string value
            let clean_value = value_str.trim_matches('"');

            writer.write_record(&[key, clean_value])?;
        }

        // Get the CSV content as a string
        let csv_content = String::from_utf8(writer.into_inner()?)?;
        Ok(csv_content)
    } else {
        // If not an object, create a CSV with just the value
        let json_str = json.to_string();
        let csv_content = format!("key,value\nvalue,{}", escape_csv_field(&json_str));
        Ok(csv_content)
    }
}

/// Convert environment variables format to CSV
fn convert_env_to_csv(text: &str) -> Result<String, Box<dyn Error>> {
    let mut writer = csv::Writer::from_writer(vec![]);

    // Write the header row
    writer.write_record(&["key", "value"])?;

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip blank lines and commented lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split by the first equals sign
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();

            // Remove quotes from value
            let processed_value = value.trim_matches('"');

            // Add to our CSV if key is not empty
            if !key.is_empty() {
                writer.write_record(&[key, processed_value])?;
            }
        }
    }

    // Get the CSV content as a string
    let csv_content = String::from_utf8(writer.into_inner()?)?;
    Ok(csv_content)
}

/// Escape a field for CSV
fn escape_csv_field(field: &str) -> String {
    let mut writer = csv::Writer::from_writer(vec![]);
    writer.write_record(&[field]).unwrap_or_default();
    String::from_utf8(writer.into_inner().unwrap_or_default()).unwrap_or_default()
}

async fn list_secrets(
    client: &Client,
    filter: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut secrets = Vec::new();
    let mut next_token = None;

    // Use pagination to retrieve all secrets
    loop {
        let mut request = client.list_secrets();

        // If we have a next token, use it to get the next page
        if let Some(token) = next_token {
            request = request.next_token(token);
        }

        let response = request.send().await?;

        // Process the current page of results
        if let Some(secret_list) = response.secret_list {
            for secret in secret_list {
                if let Some(name) = secret.name {
                    // Apply filter if provided
                    if let Some(ref f) = filter {
                        if name.contains(f) {
                            secrets.push(name);
                        }
                    } else {
                        secrets.push(name);
                    }
                }
            }
        }

        // Check if there are more pages
        next_token = response.next_token;

        // If there's no next token, we've retrieved all secrets
        if next_token.is_none() {
            break;
        }
    }

    // Sort the secrets alphabetically for better readability
    secrets.sort();

    if secrets.is_empty() {
        println!("No secrets found.");
    } else {
        println!("Available secrets:");
        for secret in &secrets {
            println!("- {}", secret);
        }
        println!("\nTotal: {} secrets", secrets.len());
    }

    Ok(())
}
