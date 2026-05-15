use crate::EventEmitter;
use crate::chaos::ChaosBackend;
use crate::kafka::KafkaBackend;
use crate::schema_registry::SchemaRegistryBackend;
use crate::seed::SeedBackend;

pub trait SiegeContext: Send + Sync + 'static {
    type Kafka: KafkaBackend;
    type Events: EventEmitter;
    type Chaos: ChaosBackend;
    type Seeder: SeedBackend;
    type SchemaRegistry: SchemaRegistryBackend;

    fn kafka(&self) -> &Self::Kafka;
    fn events(&self) -> &Self::Events;
    fn chaos(&self) -> &Self::Chaos;
    fn seeder(&self) -> &Self::Seeder;
    fn schema_registry(&self) -> Option<&Self::SchemaRegistry>;
}
