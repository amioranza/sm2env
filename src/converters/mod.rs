pub mod csv;
pub mod env;
pub mod json;
pub mod yaml;

use crate::errors::SmError;
use crate::OutputFormat;
use serde_json::{Map, Value};

pub fn convert_to_format(data: &Map<String, Value>, format: &OutputFormat) -> Result<String, SmError> {
    match format {
        OutputFormat::Stdout | OutputFormat::Env => Ok(env::convert(data)),
        OutputFormat::Json => json::convert(data),
        OutputFormat::Yaml => yaml::convert(data),
        OutputFormat::Csv => csv::convert(data),
    }
}
