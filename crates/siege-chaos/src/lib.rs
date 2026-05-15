mod error;
mod payloads;

pub use error::ChaosError;

use siege::kafka::{KafkaBackend, TopicDetail};
use siege::{ChaosBackend, KafkaProperties};

pub struct ChaosClient {
    backend: Box<dyn KafkaBackend>,
}

impl ChaosClient {
    pub fn new(backend: impl KafkaBackend) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }
}

impl ChaosBackend for ChaosClient {
    type Error = ChaosError;

    async fn get_topic(&self, name: &str) -> Result<TopicDetail, ChaosError> {
        Ok(self.backend.get_topic(name).await?)
    }

    async fn delete_topic(&self, topic: &str) -> Result<(), ChaosError> {
        Ok(self.backend.delete_topic(topic).await?)
    }

    async fn low_retention(&self, topic: &str) -> Result<(), ChaosError> {
        let config: KafkaProperties = std::collections::HashMap::from([
            ("retention.ms".into(), "1".into()),
            ("segment.ms".into(), "1".into()),
        ])
        .into();
        Ok(self.backend.update_topic_config(topic, config).await?)
    }

    async fn flip_cleanup_policy(&self, topic: &str) -> Result<(), ChaosError> {
        let detail = self.backend.get_topic(topic).await?;
        let current = detail
            .config
            .get("cleanup.policy")
            .map(|s| s.as_str())
            .unwrap_or("delete");
        let new_policy = if current.contains("compact") {
            "delete"
        } else {
            "compact"
        };
        let config: KafkaProperties =
            std::collections::HashMap::from([("cleanup.policy".into(), new_policy.into())]).into();
        Ok(self.backend.update_topic_config(topic, config).await?)
    }

    async fn increase_partitions(&self, topic: &str, extra: i32) -> Result<(), ChaosError> {
        let detail = self.backend.get_topic(topic).await?;
        let total = (detail.partitions + extra) as usize;
        Ok(self.backend.create_partitions(topic, total).await?)
    }

    async fn poison_pills(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        let producer = self.backend.producer();
        for _ in 0..count {
            let payload = payloads::poison_pill();
            producer.send(topic, &payload).await?;
        }
        Ok(())
    }

    async fn schema_break(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        let producer = self.backend.producer();
        for _ in 0..count {
            let payload = payloads::schema_breaking_json();
            producer.send(topic, &payload).await?;
        }
        Ok(())
    }
}
