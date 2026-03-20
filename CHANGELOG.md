# Changelog

## [0.2.0] - 2026-03-20

### Breaking Changes

- **CSV output format corrected**: The previous CSV output was malformed (each field was written as a complete CSV record including a trailing newline). Output is now valid RFC 4180. Downstream consumers of the old CSV format will need to update their parsers.
- **JSON format detection restricted to objects**: Secrets whose value is a JSON scalar (`"123"`, `"true"`, `"null"`) or array (`["a","b"]`) were previously misclassified as JSON. They are now treated as plain text. Any tooling relying on the old behavior must account for this change.
- **Output files written with 0600 permissions**: Secret files are now created with owner read/write only (`-rw-------`). Scripts or tools that rely on group or world readability of output files will need to adjust permissions manually or update their workflows.

### New Features

- `--region <region>`: Override the AWS region per command
- `--profile <profile>`: Select a named AWS credentials profile per command
- `--version-stage <stage>`: Fetch `AWSCURRENT` (default) or `AWSPREVIOUS`
- `--prefix <prefix>`: Prepend a string to all output keys
- `--keys <key1,key2>`: Extract only specific keys from a secret
- `--dry-run`: Print converted output to stdout without writing any file
- `--append`: Merge secret into an existing `.env` file (last-write-wins on duplicate keys)
- Multiple secret names with `--merge`: `sm2env get secret-a secret-b --merge --file .env`
- `completions <shell>`: Generate shell completions for bash, zsh, and fish
- `~/.sm2env` config file: Set default `region`, `profile`, and `format` in TOML

### Bug Fixes

- Fixed malformed CSV output: `escape_csv_field()` was returning a full CSV record instead of an escaped field value. CSV output now delegates entirely to `csv::Writer` for correct RFC 4180 escaping.
- Fixed JSON format detection: `detect_secret_format()` now only classifies secrets as JSON when the top-level value is an object. Arrays and scalars are correctly treated as plain text.
- Fixed case-insensitive `--filter` in `list` command.
- Empty secrets (`{}`) now produce an error instead of silently writing an empty file.

### Security

- Output files are created with mode `0600` (owner read/write only) on Unix platforms.
- `--file` paths containing `..` components or absolute paths outside the working directory are rejected.

### Improvements

- Modularized codebase: `src/main.rs` split into `aws_client`, `detect`, `output`, `config`, and `converters` modules.
- Replaced deprecated `serde_yaml` with `serde_yml`.
- Narrowed `tokio` features from `full` to `["macros", "rt-multi-thread"]`.
- Replaced `Box<dyn Error>` with typed `SmError` enum via `thiserror`.
- Fixed all Clippy warnings.
- Added 38 unit tests covering CSV, detection, path validation, converters, config, and more.

## [0.1.5] - Prior release

Initial public release with basic get/list functionality.
