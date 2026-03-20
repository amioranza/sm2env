# sm2env

A CLI tool to fetch AWS Secrets Manager secrets and export them in multiple formats (.env, JSON, YAML, CSV).

## Features

- Fetch secrets from AWS Secrets Manager
- Save secrets in different formats (stdout, JSON, .env, YAML, CSV)
- Write output directly to a specified file with the `--file` option
- Support for all AWS Secrets Manager formats (JSON, plain text, binary)
- List available secrets with optional case-insensitive filtering
- Stdout output uses the same KEY=VALUE format as .env files
- Per-command `--region` and `--profile` overrides
- `--dry-run` mode prints output without writing files
- `--append` mode merges secret into an existing .env file
- `--prefix` prepends a string to all output keys
- `--keys` extracts only specified keys from a secret
- `--version-stage` selects AWSCURRENT or AWSPREVIOUS
- Multiple secret fetch with `--merge`
- Shell completions for bash, zsh, and fish via `completions` subcommand
- `~/.sm2env` config file for default region, profile, and format
- Output files written with `0600` permissions (owner read/write only)

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
# Download the latest release (Intel)
curl -L https://github.com/amioranza/sm2env/releases/latest/download/sm2env-x86_64-apple-darwin.tar.gz -o sm2env.tar.gz

# Download the latest release (Apple Silicon / ARM)
curl -L https://github.com/amioranza/sm2env/releases/latest/download/sm2env-aarch64-apple-darwin.tar.gz -o sm2env.tar.gz

# Extract the binary
tar -xzf sm2env.tar.gz

# Move to a directory in your PATH
sudo mv sm2env /usr/local/bin/
```

> Check the exact filename for your version on the [Releases page](https://github.com/amioranza/sm2env/releases).

#### Windows

Download the ZIP file from the [Releases page](https://github.com/amioranza/sm2env/releases), extract it, and move `sm2env.exe` to a directory listed in your `PATH` environment variable (e.g. `C:\Windows\System32` or a custom tools directory).

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

With filtering (case-insensitive):

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

# Save as CSV file (key,value format)
sm2env get my-secret-name --output csv

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

# Write CSV output to a file
sm2env get my-secret-name --output csv --file /path/to/config.csv

# Write raw content to a file (using stdout format)
sm2env get my-secret-name --output stdout --file /path/to/output.txt
```

**Important notes about the `--file` option:**

- The `--file` option works with all output formats (`stdout`, `json`, `env`, `yaml`, `csv`)
- When using `--output stdout` with `--file`, the raw content is written to the file without affecting the original format
- The file extension is not automatically added; you must specify the complete filename
- If no `--file` option is provided, the tool behaves as before (writes to default file based on format)
- The `--file` option takes precedence over the default behavior for each output format

## Output Format Details

### ENV Format

- Default file: `.env`
- Format: `KEY=VALUE` pairs, one per line
- No quotes around values

### JSON Format

- Default file: `secret.json`
- Format: Standard JSON with pretty-printing

### YAML Format

- Default file: `secret.yaml`
- Format: Standard YAML

### CSV Format

- Default file: `secret.csv`
- Format: RFC 4180 compliant CSV with a header row (`key,value`)
- All values properly escaped according to CSV standards

### Stdout Format

- Directly prints to console
- For key-value pairs, prints in `KEY=VALUE` format
- For binary data, indicates size in bytes

## AWS Configuration

### Credentials

This tool uses the AWS SDK for Rust, which looks for credentials in the following order:

1. Environment variables: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`
2. AWS credentials file: `~/.aws/credentials`
3. IAM role for Amazon EC2 or ECS task role

Make sure you have the appropriate AWS credentials configured before using this tool.

### Advanced `get` Options

```bash
# Override region and profile per command
sm2env get my-secret --region us-west-2 --profile staging

# Fetch a specific version stage
sm2env get my-secret --version-stage AWSPREVIOUS

# Add a prefix to all keys
sm2env get my-secret --prefix APP_

# Extract only specific keys
sm2env get my-secret --keys DB_HOST,DB_PORT

# Print output without writing a file
sm2env get my-secret --dry-run

# Merge secret into existing .env (last-write-wins on duplicates)
sm2env get my-secret --append --file .env

# Fetch and merge multiple secrets
sm2env get secret-a secret-b --merge --file .env
```

### Shell Completions

```bash
# Generate completions and add to your shell config
sm2env completions bash >> ~/.bashrc
sm2env completions zsh  >> ~/.zshrc
sm2env completions fish >> ~/.config/fish/completions/sm2env.fish
```

### Config File

Create `~/.sm2env` to set defaults (TOML format):

```toml
region  = "us-east-1"
profile = "default"
format  = "env"
```

CLI flags always take precedence over config file values.

### Region

A region must be configured. You can set it via:

```bash
export AWS_DEFAULT_REGION=us-east-1
```

Or configure it in `~/.aws/config`:

```ini
[default]
region = us-east-1
```

### Required IAM Permissions

The IAM user or role must have the following permissions:

```json
{
  "Effect": "Allow",
  "Action": [
    "secretsmanager:GetSecretValue",
    "secretsmanager:ListSecrets"
  ],
  "Resource": "*"
}
```

You can scope `Resource` to specific secret ARNs to follow the principle of least privilege.

## License

MIT
