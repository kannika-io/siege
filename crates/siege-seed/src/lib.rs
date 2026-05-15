mod avro;
mod faker;

use siege::kafka::KafkaBackend;
use siege::{KafkaProperties, SeedBackend, SeedResult, SiegeError};

pub struct TopicSeed {
    name: String,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
}

impl TopicSeed {
    pub fn new(name: &str, partitions: i32) -> Self {
        Self {
            name: name.to_owned(),
            partitions,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }
    }

    pub fn config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

pub struct Seeder {
    backend: Box<dyn KafkaBackend>,
    seeds: Vec<TopicSeed>,
}

impl Seeder {
    pub fn new(backend: impl KafkaBackend) -> Self {
        Self {
            backend: Box::new(backend),
            seeds: Vec::new(),
        }
    }

    pub fn topic(mut self, seed: TopicSeed) -> Self {
        self.seeds.push(seed);
        self
    }
}

impl SeedBackend for Seeder {
    type Error = SiegeError;

    async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        let mut created = Vec::new();
        let mut skipped = Vec::new();

        for seed in &self.seeds {
            match self
                .backend
                .create_topic(
                    &seed.name,
                    seed.partitions,
                    seed.replication_factor,
                    seed.config.clone(),
                )
                .await
            {
                Ok(()) => created.push(seed.name.clone()),
                Err(_) => skipped.push(seed.name.clone()),
            }
        }

        Ok(SeedResult { created, skipped })
    }
}
