use crate::EventEmitter;
use crate::kafka::KafkaBackend;

pub trait SiegeContext: Send + Sync + 'static {
    type Kafka: KafkaBackend;
    type Events: EventEmitter;

    fn kafka(&self) -> &Self::Kafka;
    fn events(&self) -> &Self::Events;
}
