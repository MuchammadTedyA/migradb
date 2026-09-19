use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid migration version: {0}")]
    InvalidVersion(String),

    #[error("Migration already applied: {0}")]
    AlreadyApplied(String),

    #[error("Migration not found: {0}")]
    NotFound(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Codegen error: {0}")]
    Codegen(String),
}

pub type Result<T> = std::result::Result<T, MigrationError>;
