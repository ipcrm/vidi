use serde::{Serialize, Serializer};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("http: {0}")]
    Http(String),

    #[error("url: {0}")]
    Url(#[from] url::ParseError),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("sanitize: {0}")]
    Sanitize(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(format!("{e:#}"))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            kind: &'a str,
            message: String,
        }
        let kind = match self {
            AppError::Io(_) => "io",
            AppError::Http(_) => "http",
            AppError::Url(_) => "url",
            AppError::Json(_) => "json",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidArgument(_) => "invalid_argument",
            AppError::Sanitize(_) => "sanitize",
            AppError::Internal(_) => "internal",
        };
        Wire {
            kind,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Small helper for constructing not-found errors with a path context.
pub fn not_found(what: impl fmt::Display) -> AppError {
    AppError::NotFound(what.to_string())
}
