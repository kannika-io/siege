use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChaosError {
    #[error("topic not found: {0}")]
    TopicNotFound(String),
    #[error("kafka error: {0}")]
    KafkaError(String),
}

impl From<siege::SiegeError> for ChaosError {
    fn from(e: siege::SiegeError) -> Self {
        match e {
            siege::SiegeError::TopicNotFound(s) => ChaosError::TopicNotFound(s),
            _ => ChaosError::KafkaError(e.to_string()),
        }
    }
}
