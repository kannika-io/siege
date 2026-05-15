mod avro;
mod faker;

pub use siege_seed_avsc::avsc;

use apache_avro::Schema;
use fake::rand::SeedableRng;
use fake::rand::rngs::StdRng;
use siege::kafka::KafkaBackend;
use siege::schema_registry::SchemaRegistryBackend;
use siege::{KafkaProperties, SeedBackend, SeedResult, SiegeError};

use avro::AvroSerializer;
use faker::generate_record;

pub struct TopicSeed {
    name: String,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
    schema: Option<&'static str>,
    record_count: Option<u32>,
}

impl TopicSeed {
    pub fn new(name: &str, partitions: i32) -> Self {
        Self {
            name: name.to_owned(),
            partitions,
            replication_factor: 1,
            config: KafkaProperties::new(),
            schema: None,
            record_count: None,
        }
    }

    pub fn config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    pub fn schema(mut self, schema: &'static str) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn records(mut self, count: u32) -> Self {
        self.record_count = Some(count);
        self
    }
}

pub struct Seeder {
    kafka: Box<dyn KafkaBackend>,
    schema_registry: Option<Box<dyn SchemaRegistryBackend>>,
    seeds: Vec<TopicSeed>,
    rng_seed: u64,
}

impl Seeder {
    pub fn new(kafka: impl KafkaBackend) -> Self {
        Self {
            kafka: Box::new(kafka),
            schema_registry: None,
            seeds: Vec::new(),
            rng_seed: 42,
        }
    }

    pub fn schema_registry(mut self, sr: impl SchemaRegistryBackend) -> Self {
        self.schema_registry = Some(Box::new(sr));
        self
    }

    pub fn topic(mut self, seed: TopicSeed) -> Self {
        self.seeds.push(seed);
        self
    }

    pub fn rng_seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }

    async fn seed_data(
        &self,
        topic: &str,
        schema_str: &str,
        count: u32,
        rng: &mut StdRng,
    ) -> Result<(), SiegeError> {
        let sr = self
            .schema_registry
            .as_ref()
            .ok_or_else(|| SiegeError::Seed("schema_registry is required to seed data".into()))?;

        let subject = format!("{topic}-value");
        let schema_id = sr.register_schema(&subject, schema_str).await?;

        let schema = Schema::parse_str(schema_str)
            .map_err(|e| SiegeError::Seed(format!("failed to parse schema: {e}")))?;

        let serializer = AvroSerializer::new(schema.clone(), schema_id);
        let producer = self.kafka.producer();

        let namespace = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, topic.as_bytes());

        let mut records = Vec::with_capacity(count as usize);

        for i in 0..count {
            let record = generate_record(&schema, rng)?;
            let payload = serializer.serialize(record)?;
            let key = uuid::Uuid::new_v5(&namespace, &i.to_be_bytes());
            records.push((key, payload));
        }

        let futures: Vec<_> = records
            .iter()
            .map(|(key, payload)| producer.send(topic, Some(key.as_bytes()), payload))
            .collect();

        futures::future::try_join_all(futures).await?;

        Ok(())
    }
}

impl SeedBackend for Seeder {
    type Error = SiegeError;

    async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        let mut created = Vec::new();
        let mut skipped = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        for seed in &self.seeds {
            match self
                .kafka
                .create_topic(
                    &seed.name,
                    seed.partitions,
                    seed.replication_factor,
                    seed.config.clone(),
                )
                .await
            {
                Ok(()) => {
                    created.push(seed.name.clone());

                    if let (Some(schema_str), Some(count)) = (seed.schema, seed.record_count) {
                        self.seed_data(&seed.name, schema_str, count, &mut rng)
                            .await?;
                    }
                }
                Err(_) => skipped.push(seed.name.clone()),
            }
        }

        Ok(SeedResult { created, skipped })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siege::{MockKafkaBackend, NoopSchemaRegistry};

    #[tokio::test]
    async fn seed_creates_topics_and_reports_result() {
        let kafka = MockKafkaBackend::new();
        let seeder = Seeder::new(kafka)
            .topic(TopicSeed::new("topic-a", 3))
            .topic(TopicSeed::new("topic-b", 1));

        let result = seeder.seed_topics().await.expect("should seed");
        assert_eq!(result.created, vec!["topic-a", "topic-b"]);
        assert!(result.skipped.is_empty());
    }

    #[tokio::test]
    async fn seed_skips_existing_topics() {
        let kafka = MockKafkaBackend::with_topics(vec![
            siege::kafka::TopicDetail {
                name: "existing".into(),
                partitions: 3,
                replication_factor: 1,
                config: KafkaProperties::new(),
            },
        ]);
        let seeder = Seeder::new(kafka)
            .topic(TopicSeed::new("existing", 3))
            .topic(TopicSeed::new("new-one", 1));

        let result = seeder.seed_topics().await.expect("should seed");
        assert_eq!(result.created, vec!["new-one"]);
        assert_eq!(result.skipped, vec!["existing"]);
    }

    #[tokio::test]
    async fn seed_with_schema_and_records_produces_data() {
        let kafka = MockKafkaBackend::new();
        let schema_registry = NoopSchemaRegistry;
        let schema_str = r#"{"type":"record","name":"Test","namespace":"io.siege.schemas","fields":[{"name":"name","type":"string"}]}"#;

        let seeder = Seeder::new(kafka)
            .schema_registry(schema_registry)
            .topic(
                TopicSeed::new("test-topic", 1)
                    .schema(schema_str)
                    .records(10),
            );

        let result = seeder.seed_topics().await.expect("should seed");
        assert_eq!(result.created, vec!["test-topic"]);
    }

    #[tokio::test]
    async fn seed_deterministic_with_same_rng_seed() {
        let schema_str = r#"{"type":"record","name":"Test","namespace":"io.siege.schemas","fields":[{"name":"name","type":"string"},{"name":"age","type":"int"}]}"#;

        let kafka1 = MockKafkaBackend::new();
        let sr1 = NoopSchemaRegistry;
        let seeder1 = Seeder::new(kafka1)
            .schema_registry(sr1)
            .rng_seed(123)
            .topic(TopicSeed::new("t", 1).schema(schema_str).records(5));

        let kafka2 = MockKafkaBackend::new();
        let sr2 = NoopSchemaRegistry;
        let seeder2 = Seeder::new(kafka2)
            .schema_registry(sr2)
            .rng_seed(123)
            .topic(TopicSeed::new("t", 1).schema(schema_str).records(5));

        let r1 = seeder1.seed_topics().await.expect("should seed");
        let r2 = seeder2.seed_topics().await.expect("should seed");
        assert_eq!(r1.created, r2.created);
    }
}
