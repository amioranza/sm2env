# sm2env

A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.

## Features

- Fetch secrets from AWS Secrets Manager
- Save secrets in different formats (stdout, JSON, .env, YAML)
- List available secrets with optional filtering
- Stdout output uses the same KEY=VALUE format as .env files

## Installation

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

## AWS Configuration

This tool uses the AWS SDK for Rust, which looks for credentials in the following order:

1. Environment variables: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`
2. AWS credentials file: `~/.aws/credentials`
3. IAM role for Amazon EC2 or ECS task role

Make sure you have the appropriate AWS credentials configured before using this tool.

## License

MIT
