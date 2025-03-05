use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::fmt;
use std::fs;

#[derive(Parser)]
#[command(
    name = "sm2env",
    about = "A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.",
    version = "0.1.1",
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

        /// Specify the output format (stdout, json, env, yaml)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Env)]
        output: OutputFormat,
    },
    /// List all available secrets
    List {
        /// Filter secrets by a specific text
        #[arg(short, long)]
        filter: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Stdout,
    Json,
    Env,
    Yaml,
}

// Implement Display for OutputFormat
impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Stdout => write!(f, "stdout"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Env => write!(f, "env"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Use the latest behavior version for AWS SDK
    let config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;
    let client = Client::new(&config);

    match &cli.command {
        Some(Commands::Get {
            secret_name,
            output,
        }) => get_secret(&client, secret_name, output).await?,
        Some(Commands::List { filter }) => list_secrets(&client, filter.clone()).await?,
        None => {
            // If no arguments are provided, print the help message
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }
    
    Ok(())
}

async fn get_secret(client: &Client, secret_name: &str, output_format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    
    if let Some(secret) = response.secret_string {
        match serde_json::from_str::<serde_json::Value>(&secret) {
            Ok(json) => {
                // Format output based on chosen format
                match output_format {
                    OutputFormat::Stdout => {
                        // Display to stdout in .env format
                        if let Some(obj) = json.as_object() {
                            for (key, value) in obj {
                                println!("{}={}", key, value);
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
                                env_content.push_str(&format!("{}={}\n", key, value));
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
                }
            }
            Err(_) => println!("The secret is not in valid JSON format."),
        }
    } else {
        println!("No secret string found in the response.");
    }
    
    Ok(())
}

async fn list_secrets(client: &Client, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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
