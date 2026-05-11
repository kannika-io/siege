use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use siege::SiegeError;

#[derive(Clone)]
pub struct Producer {
    producer: Arc<FutureProducer>,
}

impl Producer {
    pub fn new(bootstrap_servers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("failed to create Kafka producer");
        Self {
            producer: Arc::new(producer),
        }
    }

    pub async fn send(&self, topic: &str, payload: &[u8]) -> Result<(), SiegeError> {
        let record = FutureRecord::<(), [u8]>::to(topic).payload(payload);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| SiegeError::KafkaError(e.to_string()))?;
        Ok(())
    }
}
