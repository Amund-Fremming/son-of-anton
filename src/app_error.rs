use std::env::VarError;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing env var: {0}")]
    MissingVar(#[from] VarError),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Http error: {0} - {1}")]
    Http(StatusCode, String),

    #[error("Standard library error: {0}")]
    Std(#[from] std::io::Error),
}
