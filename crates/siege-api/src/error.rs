use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub struct HttpError {
    pub status: StatusCode,
    pub body: serde_json::Value,
}

impl HttpError {
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: serde_json::json!({ "error": "NotFound", "detail": detail.into() }),
        }
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: serde_json::json!({ "error": "Conflict", "detail": detail.into() }),
        }
    }

    pub fn bad_gateway(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            body: serde_json::json!({ "error": "BadGateway", "detail": detail.into() }),
        }
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: serde_json::json!({ "error": "Internal", "detail": detail.into() }),
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)
    }
}

impl ResponseError for HttpError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(&self.body)
    }
}

impl From<siege::SiegeError> for HttpError {
    fn from(e: siege::SiegeError) -> Self {
        match e {
            siege::SiegeError::TopicNotFound(s) => Self::not_found(s),
            siege::SiegeError::TopicAlreadyExists(s) => Self::conflict(s),
            siege::SiegeError::KafkaError(s) => Self::bad_gateway(s),
            siege::SiegeError::Chaos(s) => Self::bad_gateway(s),
            siege::SiegeError::Seed(s) => Self::bad_gateway(s),
            siege::SiegeError::Internal(s) => Self::internal(s),
        }
    }
}

