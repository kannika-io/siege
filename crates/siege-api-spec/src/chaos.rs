use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosTopicRequest {
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosPartitionsRequest {
    pub topic: String,
    pub partitions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosProduceRequest {
    pub topic: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosResult {
    pub topic: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum ChaosErrorResponse {
    TopicNotFound { message: String },
    KafkaError { message: String },
    ProducerError { message: String },
}
