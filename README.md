# sm2env

A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.

## Features

- Fetch secrets from AWS Secrets Manager
- Save secrets in different formats (stdout, JSON, .env, YAML)
- Write output directly to a specified file with the `--file` option
- Support for all AWS Secrets Manager formats (JSON, plain text, binary)
- List available secrets with optional filtering
- Stdout output uses the same KEY=VALUE format as .env files

## Installation

### Direct Installation (macOS and Linux)

```bash
# One-line installation
curl -fsSL https://raw.githubusercontent.com/amioranza/sm2env/main/install.sh | bash
```

### Using Homebrew (macOS and Linux)

```bash
# Add the tap
brew tap amioranza/tools

# Install sm2env
brew install sm2env
```

### From Releases

You can download pre-built binaries from the [GitHub Releases page](https://github.com/amioranza/sm2env/releases).

#### Linux
```bash
# Download the latest release
curl -L https://github.com/amioranza/sm2env/releases/latest/download/sm2env-v*-x86_64-linux.tar.gz -o sm2env.tar.gz

# Extract the binary
tar -xzf sm2env.tar.gz

# Move to a directory in your PATH
sudo mv sm2env /usr/local/bin/
```

#### macOS
```bash
# Download the latest release
curl -L https://github.com/amioranza/sm2env/releases/latest/download/sm2env-v*-x86_64-apple-darwin.tar.gz -o sm2env.tar.gz

# Extract the binary
tar -xzf sm2env.tar.gz

# Move to a directory in your PATH
sudo mv sm2env /usr/local/bin/
```

#### Windows
Download the ZIP file from the [Releases page](https://github.com/amioranza/sm2env/releases) and extract it to a location in your PATH.

### From Source

Make sure you have Rust and Cargo installed. Then, you can build the project:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/sm2env`.

## Usage

### List available secrets

```bash
sm2env list
```

With filtering:

```bash
sm2env list --filter dev
```

### Get a secret

Retrieve a secret and save it as a .env file (default):

```bash
sm2env get my-secret-name
```

Specify a different output format:

```bash
# Print to stdout in KEY=VALUE format
sm2env get my-secret-name --output stdout

# Save as JSON file
sm2env get my-secret-name --output json

# Save as YAML file
sm2env get my-secret-name --output yaml

# Save as .env file (default)
sm2env get my-secret-name --output env
```

### Write output to a specific file

You can use the `--file` option to write the output directly to a specified file path:

```bash
# Write to a specific .env file
sm2env get my-secret-name --output env --file /path/to/my-custom.env

# Write JSON output to a file
sm2env get my-secret-name --output json --file /path/to/config.json

# Write YAML output to a file
sm2env get my-secret-name --output yaml --file /path/to/config.yaml

# Write raw content to a file (using stdout format)
sm2env get my-secret-name --output stdout --file /path/to/output.txt
```

**Important notes about the `--file` option:**

- The `--file` option works with all output formats (`stdout`, `json`, `env`, `yaml`)
- When using `--output stdout` with `--file`, the raw content is written to the file without affecting the original format
- The file extension is not automatically added; you must specify the complete filename
- If no `--file` option is provided, the tool behaves as before (writes to `.env` file or stdout)
- The `--file` option takes precedence over the default behavior for each output format

## AWS Configuration

This tool uses the AWS SDK for Rust, which looks for credentials in the following order:

1. Environment variables: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`
2. AWS credentials file: `~/.aws/credentials`
3. IAM role for Amazon EC2 or ECS task role

Make sure you have the appropriate AWS credentials configured before using this tool.

## License

MIT
