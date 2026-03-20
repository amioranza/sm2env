use crate::errors::SmError;
use std::path::Path;

/// Validate that a path is safe to write to.
/// Rejects paths containing `..` components or absolute paths outside cwd.
pub fn validate_path(path: &Path) -> Result<(), SmError> {
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(SmError::PathError(format!(
                "Path '{}' contains '..' components which are not allowed",
                path.display()
            )));
        }
    }

    if path.is_absolute() {
        let cwd = std::env::current_dir()?;
        if !path.starts_with(&cwd) {
            return Err(SmError::PathError(format!(
                "Absolute path '{}' is outside the working directory",
                path.display()
            )));
        }
    }

    Ok(())
}

/// Write content to the given path with restricted permissions (0600 on Unix),
/// or print to stdout if path is None.
pub fn write_output(content: &str, path: Option<&Path>) -> Result<(), SmError> {
    match path {
        None => {
            print!("{}", content);
            Ok(())
        }
        Some(p) => {
            validate_path(p)?;
            write_secure(p, content)
        }
    }
}

#[cfg(unix)]
fn write_secure(path: &Path, content: &str) -> Result<(), SmError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(not(unix))]
fn write_secure(path: &Path, content: &str) -> Result<(), SmError> {
    std::fs::write(path, content)?;
    Ok(())
}
