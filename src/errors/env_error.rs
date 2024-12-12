use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvironmentError {
    #[error("Environment not found: {0}")]
    NotFound(String),

    #[error("Environment with duplicate name: {0}")]
    DuplicateName(String),

    #[error("Invalid environment index: {0}")]
    InvalidIndex(usize),
}
