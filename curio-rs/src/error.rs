pub type Result<T> = anyhow::Result<T>;

/// A user-correctable CLI input error with a stable machine-readable code.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct CliValidationError {
    pub code: &'static str,
    pub message: String,
    pub hint: &'static str,
}

impl CliValidationError {
    pub fn new(code: &'static str, message: impl Into<String>, hint: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
            hint,
        }
    }
}
