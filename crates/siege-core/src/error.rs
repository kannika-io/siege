use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "error", content = "detail")]
pub enum SiegeError {
    #[error("topic not found: {0}")]
    TopicNotFound(String),
    #[error("topic already exists: {0}")]
    TopicAlreadyExists(String),
    #[error("kafka error: {0}")]
    KafkaError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serializes_with_tag() {
        let err = SiegeError::TopicNotFound("my-topic".into());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"error\":\"TopicNotFound\""));
        assert!(json.contains("\"detail\":\"my-topic\""));
    }
}
