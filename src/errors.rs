use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum SmError {
    #[error("AWS error: {0}")]
    AwsError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Path error: {0}")]
    PathError(String),
}

impl From<serde_json::Error> for SmError {
    fn from(e: serde_json::Error) -> Self {
        SmError::ParseError(e.to_string())
    }
}

impl From<serde_yml::Error> for SmError {
    fn from(e: serde_yml::Error) -> Self {
        SmError::FormatError(e.to_string())
    }
}

impl From<csv::Error> for SmError {
    fn from(e: csv::Error) -> Self {
        SmError::FormatError(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for SmError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        SmError::FormatError(e.to_string())
    }
}

impl From<toml::de::Error> for SmError {
    fn from(e: toml::de::Error) -> Self {
        SmError::ParseError(e.to_string())
    }
}
