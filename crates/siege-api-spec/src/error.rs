use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum ListTopicsError {
    KafkaError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum GetTopicError {
    TopicNotFound { message: String },
    KafkaError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum CreateTopicError {
    TopicAlreadyExists { message: String },
    KafkaError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum DeleteTopicError {
    TopicNotFound { message: String },
    KafkaError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum UpdateTopicConfigError {
    TopicNotFound { message: String },
    KafkaError { message: String },
}
