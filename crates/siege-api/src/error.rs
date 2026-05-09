use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use siege_api_spec::SiegeError;
use std::fmt;

#[derive(Debug)]
pub struct ApiError(pub SiegeError);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status = match &self.0 {
            SiegeError::TopicNotFound(_) => StatusCode::NOT_FOUND,
            SiegeError::TopicAlreadyExists(_) => StatusCode::CONFLICT,
            SiegeError::KafkaError(_) => StatusCode::BAD_GATEWAY,
            SiegeError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        HttpResponse::build(status).json(&self.0)
    }
}

impl From<SiegeError> for ApiError {
    fn from(e: SiegeError) -> Self {
        ApiError(e)
    }
}
