# sm2env

A CLI tool to fetch AWS Secrets Manager secrets and save them as .env files.

## Features

- Fetch secrets from AWS Secrets Manager
- Save secrets in different formats (stdout, JSON, .env, YAML)
- List available secrets with optional filtering
- Stdout output uses the same KEY=VALUE format as .env files

## Installation

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
