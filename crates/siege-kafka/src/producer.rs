use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use siege::kafka::{BoxFuture, KafkaProducer};
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
}

impl KafkaProducer for Producer {
    fn send<'a>(
        &'a self,
        topic: &'a str,
        key: Option<&'a [u8]>,
        payload: &'a [u8],
    ) -> BoxFuture<'a, Result<(), SiegeError>> {
        Box::pin(async move {
            let mut record = FutureRecord::<[u8], [u8]>::to(topic).payload(payload);
            if let Some(k) = key {
                record = record.key(k);
            }
            self.producer
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(e, _)| SiegeError::KafkaError(e.to_string()))?;
            Ok(())
        })
    }
}
