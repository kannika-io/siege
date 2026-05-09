use siege_core::kafka::KafkaBackend;

use crate::sse::broadcaster::Broadcaster;

pub trait SiegeContext: Send + Sync + 'static {
    type Kafka: KafkaBackend;

    fn kafka(&self) -> &Self::Kafka;
    fn events(&self) -> &Broadcaster;
}
