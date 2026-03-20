mod aws_client;
mod config;
mod converters;
mod detect;
mod errors;
mod output;
mod tests;

use base64::Engine;

struct GetOptions<'a> {
    secret_names: &'a [String],
    output_format: &'a OutputFormat,
    file: Option<&'a str>,
    version_stage: &'a str,
    prefix: Option<&'a str>,
    keys: Option<&'a str>,
    dry_run: bool,
    append: bool,
    merge: bool,
}
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use detect::{detect_secret_format, parse_env_vars, secret_to_map};
use errors::SmError;
use serde_json::{Map, Value};
use std::fmt;
use std::io;

#[derive(Parser)]
#[command(
    name = "sm2env",
    about = "A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.",
    version = "0.2.0",
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
    /// Fetch one or more secrets and save them in the specified format
    Get {
        /// One or more secret names to retrieve
        #[arg(required = true)]
        secret_names: Vec<String>,

        /// Output format (stdout, json, env, yaml, csv)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Env)]
        output: OutputFormat,

        /// File path to write the output to
        #[arg(short, long)]
        file: Option<String>,

        /// Override the AWS region
        #[arg(long)]
        region: Option<String>,

        /// Use a named AWS credentials profile
        #[arg(long)]
        profile: Option<String>,

        /// Secret version stage (default: AWSCURRENT)
        #[arg(long, default_value = "AWSCURRENT")]
        version_stage: String,

        /// Prepend a prefix to all output keys
        #[arg(long)]
        prefix: Option<String>,

        /// Extract only specific keys (comma-separated)
        #[arg(long)]
        keys: Option<String>,

        /// Print output to stdout without writing any file
        #[arg(long)]
        dry_run: bool,

        /// Merge secret into existing .env file (last-write-wins on duplicates)
        #[arg(long)]
        append: bool,

        /// Merge multiple secrets into one output (required when >1 secret name)
        #[arg(long)]
        merge: bool,
    },
    /// List all available secrets
    List {
        /// Filter secrets by name (case-insensitive)
        #[arg(short, long)]
        filter: Option<String>,

        /// Override the AWS region
        #[arg(long)]
        region: Option<String>,

        /// Use a named AWS credentials profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
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

    match &cli.command {
        Some(Commands::Get {
            secret_names,
            output,
            file,
            region,
            profile,
            version_stage,
            prefix,
            keys,
            dry_run,
            append,
            merge,
        }) => {
            // Load config file, CLI flags take precedence
            let cfg = config::load_config()?;
            let effective_region = region.as_deref().or(cfg.region.as_deref());
            let effective_profile = profile.as_deref().or(cfg.profile.as_deref());

            let client = aws_client::build_client(effective_region, effective_profile).await;

            get_secret(
                &client,
                &GetOptions {
                    secret_names,
                    output_format: output,
                    file: file.as_deref(),
                    version_stage,
                    prefix: prefix.as_deref(),
                    keys: keys.as_deref(),
                    dry_run: *dry_run,
                    append: *append,
                    merge: *merge,
                },
            )
            .await?;
        }
        Some(Commands::List { filter, region, profile }) => {
            let cfg = config::load_config()?;
            let effective_region = region.as_deref().or(cfg.region.as_deref());
            let effective_profile = profile.as_deref().or(cfg.profile.as_deref());
            let client = aws_client::build_client(effective_region, effective_profile).await;
            list_secrets(&client, filter.as_deref()).await?;
        }
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "sm2env", &mut io::stdout());
        }
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}

async fn get_secret(
    client: &aws_sdk_secretsmanager::Client,
    opts: &GetOptions<'_>,
) -> Result<(), SmError> {
    let secret_names = opts.secret_names;
    let output_format = opts.output_format;
    let file = opts.file;
    let version_stage = opts.version_stage;
    let prefix = opts.prefix;
    let keys = opts.keys;
    let dry_run = opts.dry_run;
    let append = opts.append;
    let merge = opts.merge;

    if secret_names.len() > 1 && !merge {
        return Err(SmError::FormatError(
            "Multiple secret names provided without --merge flag. Use --merge to combine secrets."
                .to_string(),
        ));
    }

    // Fetch and merge all secrets
    let mut merged_map: Map<String, Value> = Map::new();

    for secret_name in secret_names {
        let response = client
            .get_secret_value()
            .secret_id(secret_name)
            .version_stage(version_stage)
            .send()
            .await
            .map_err(|e| SmError::AwsError(e.to_string()))?;

        let map = if let Some(secret_string) = response.secret_string {
            let fmt = detect_secret_format(&secret_string);
            secret_to_map(fmt)
        } else if let Some(secret_binary) = response.secret_binary {
            let base64_str =
                base64::engine::general_purpose::STANDARD.encode(secret_binary.as_ref());
            let mut map = Map::new();
            map.insert("binary_data".to_string(), Value::String(base64_str));
            map
        } else {
            return Err(SmError::FormatError(
                "No secret content found in the response.".to_string(),
            ));
        };

        merged_map.extend(map);
    }

    // Warn/error on empty secret
    if merged_map.is_empty() {
        return Err(SmError::FormatError(
            "Secret contains no key-value pairs (empty object {}).".to_string(),
        ));
    }

    // Apply --keys filter
    if let Some(keys_str) = keys {
        let requested: Vec<&str> = keys_str.split(',').map(|k| k.trim()).collect();
        let mut filtered = Map::new();
        for key in &requested {
            if let Some(val) = merged_map.remove(*key) {
                filtered.insert(key.to_string(), val);
            } else {
                eprintln!("Warning: key '{}' not found in secret", key);
            }
        }
        merged_map = filtered;
    }

    // Apply --prefix
    if let Some(pfx) = prefix {
        let prefixed: Map<String, Value> = merged_map
            .into_iter()
            .map(|(k, v)| (format!("{}{}", pfx, k), v))
            .collect();
        merged_map = prefixed;
    }

    // Handle --append: merge into existing .env file
    let effective_map = if append {
        let target = file.unwrap_or(".env");
        let mut existing: Map<String, Value> = if std::path::Path::new(target).exists() {
            let content = std::fs::read_to_string(target)?;
            parse_env_vars(&content)
        } else {
            Map::new()
        };
        existing.extend(merged_map);
        existing
    } else {
        merged_map
    };

    // Convert to output format
    let content = converters::convert_to_format(&effective_map, output_format)?;

    // Determine output destination
    if dry_run {
        print!("{}", content);
        return Ok(());
    }

    let output_path: Option<std::path::PathBuf> = if matches!(output_format, OutputFormat::Stdout) && file.is_none() {
        None
    } else {
        let p = file.map(|f| f.to_string()).unwrap_or_else(|| {
            match output_format {
                OutputFormat::Json => "secret.json".to_string(),
                OutputFormat::Yaml => "secret.yaml".to_string(),
                OutputFormat::Csv => "secret.csv".to_string(),
                _ => ".env".to_string(),
            }
        });
        Some(std::path::PathBuf::from(p))
    };

    output::write_output(&content, output_path.as_deref())?;

    if let Some(ref p) = output_path {
        println!("Secret written to: {}", p.display());
    }

    Ok(())
}

async fn list_secrets(
    client: &aws_sdk_secretsmanager::Client,
    filter: Option<&str>,
) -> Result<(), SmError> {
    let mut secrets = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let mut request = client.list_secrets();

        if let Some(token) = next_token {
            request = request.next_token(token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| SmError::AwsError(e.to_string()))?;

        if let Some(secret_list) = response.secret_list {
            secrets.extend(
                secret_list
                    .into_iter()
                    .filter_map(|s| s.name)
                    .filter(|name| {
                        filter
                            .map(|f| name.to_lowercase().contains(&f.to_lowercase()))
                            .unwrap_or(true)
                    }),
            );
        }

        next_token = response.next_token;
        if next_token.is_none() {
            break;
        }
    }

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
