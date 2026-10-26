use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Ad not found: {0}")]
    NotFound(String),

    #[error("Ad filtered: {0}")]
    Filtered(String),

    #[error("Frequency cap exceeded: {0}")]
    FrequencyCap(String),

    #[error("Budget exhausted: {0}")]
    BudgetExhausted(String),

    #[error("ML model error: {0}")]
    Model(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorPayload {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "ValidationError"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "AdNotFoundError"),
            AppError::Filtered(_) => (StatusCode::OK, "AdFilteredError"),
            AppError::FrequencyCap(_) => (StatusCode::BAD_REQUEST, "FrequencyCapError"),
            AppError::BudgetExhausted(_) => (StatusCode::BAD_REQUEST, "BudgetExhaustedError"),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "ConfigError"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DatabaseError"),
            AppError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CacheError"),
            AppError::Model(_) => (StatusCode::INTERNAL_SERVER_ERROR, "ModelError"),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "SerializationError"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError"),
        };

        let payload = ErrorPayload {
            error: error_type.to_string(),
            message: self.to_string(),
            details: None,
            request_id: None,
        };

        (status, Json(payload)).into_response()
    }
}
