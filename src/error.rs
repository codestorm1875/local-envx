use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvxError {
    #[error("invalid key `{0}`")]
    InvalidKey(String),
    #[error("missing .env file in the current directory; run `envx init` to create one")]
    MissingEnvFile,
    #[error("key `{0}` not found")]
    KeyNotFound(String),
}
