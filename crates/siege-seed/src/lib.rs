use siege::kafka::KafkaBackend;
use siege::{KafkaProperties, SeedBackend, SeedResult, SiegeError};

struct TopicSeed {
    name: &'static str,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
}

impl TopicSeed {
    fn new(name: &'static str, partitions: i32) -> Self {
        Self {
            name,
            partitions,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }
    }

    fn config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

fn topic_seeds() -> Vec<TopicSeed> {
    vec![
        TopicSeed::new("kings-landing", 6),
        TopicSeed::new("winterfell", 3),
        TopicSeed::new("the-wall", 1),
        TopicSeed::new("iron-islands", 3),
        TopicSeed::new("dragonstone", 3),
        TopicSeed::new("the-citadel", 1).config("cleanup.policy", "compact"),
    ]
}

pub struct Seeder {
    backend: Box<dyn KafkaBackend>,
}

impl Seeder {
    pub fn new(backend: impl KafkaBackend) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }
}

impl SeedBackend for Seeder {
    type Error = SiegeError;

    async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        let mut created = Vec::new();
        let mut skipped = Vec::new();

        for seed in topic_seeds() {
            match self
                .backend
                .create_topic(
                    seed.name,
                    seed.partitions,
                    seed.replication_factor,
                    seed.config,
                )
                .await
            {
                Ok(()) => created.push(seed.name.to_owned()),
                Err(_) => skipped.push(seed.name.to_owned()),
            }
        }

        Ok(SeedResult { created, skipped })
    }
}
