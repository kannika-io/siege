mod error;
mod payloads;

pub use error::ChaosError;

use rdkafka::admin::{AdminOptions, NewPartitions};
use siege::KafkaProperties;
use siege::kafka::KafkaBackend;
use siege_kafka::{Producer, RdKafkaBackend};

pub struct ChaosClient {
    backend: RdKafkaBackend,
    producer: Producer,
}

impl ChaosClient {
    pub fn new(bootstrap_servers: &str) -> Self {
        Self {
            backend: RdKafkaBackend::new(bootstrap_servers),
            producer: Producer::new(bootstrap_servers),
        }
    }

    pub async fn get_topic(
        &self,
        name: &str,
    ) -> Result<siege::kafka::TopicDetail, ChaosError> {
        Ok(self.backend.get_topic(name).await?)
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<(), ChaosError> {
        self.backend.delete_topic(topic).await?;
        Ok(())
    }

    pub async fn zero_retention(&self, topic: &str) -> Result<(), ChaosError> {
        let config: KafkaProperties =
            std::collections::HashMap::from([("retention.ms".into(), "0".into())]).into();
        self.backend.update_topic_config(topic, config).await?;
        Ok(())
    }

    pub async fn flip_cleanup_policy(&self, topic: &str) -> Result<(), ChaosError> {
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
        let config: KafkaProperties = std::collections::HashMap::from([(
            "cleanup.policy".into(),
            new_policy.into(),
        )])
        .into();
        self.backend.update_topic_config(topic, config).await?;
        Ok(())
    }

    pub async fn increase_partitions(
        &self,
        topic: &str,
        partitions: i32,
    ) -> Result<(), ChaosError> {
        let new_parts = NewPartitions::new(topic, partitions as usize);
        self.backend
            .admin()
            .create_partitions(&[new_parts], &AdminOptions::new())
            .await
            .map_err(|e| ChaosError::KafkaError(e.to_string()))?;
        Ok(())
    }

    pub async fn poison_pills(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        for _ in 0..count {
            let payload = payloads::poison_pill();
            self.producer
                .send(topic, &payload)
                .await
                .map_err(|e| ChaosError::ProducerError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn schema_break(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        for _ in 0..count {
            let payload = payloads::schema_breaking_json();
            self.producer
                .send(topic, &payload)
                .await
                .map_err(|e| ChaosError::ProducerError(e.to_string()))?;
        }
        Ok(())
    }
}
