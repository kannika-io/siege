use std::collections::HashMap;

use siege::kafka::KafkaBackend;
use siege::KafkaProperties;

struct Seed {
    name: &'static str,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
}

fn seeds() -> Vec<Seed> {
    vec![
        Seed {
            name: "kings-landing",
            partitions: 6,
            replication_factor: 1,
            config: KafkaProperties::new(),
        },
        Seed {
            name: "winterfell",
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        },
        Seed {
            name: "the-wall",
            partitions: 1,
            replication_factor: 1,
            config: KafkaProperties::new(),
        },
        Seed {
            name: "iron-islands",
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        },
        Seed {
            name: "dragonstone",
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        },
        Seed {
            name: "the-citadel",
            partitions: 1,
            replication_factor: 1,
            config: HashMap::from([("cleanup.policy".into(), "compact".into())]).into(),
        },
    ]
}

pub async fn seed_topics(backend: &impl KafkaBackend) {
    for seed in seeds() {
        match backend
            .create_topic(seed.name, seed.partitions, seed.replication_factor, seed.config)
            .await
        {
            Ok(()) => eprintln!("seeded topic: {}", seed.name),
            Err(e) => eprintln!("seed {}: {e}", seed.name),
        }
    }
}
