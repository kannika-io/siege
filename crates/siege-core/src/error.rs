use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
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
