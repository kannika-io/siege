use serde::{Deserialize, Serialize};
use siege_kernel::KafkaProperties;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: KafkaProperties,
}

pub struct TopicCreated {
    pub topic: Topic,
}

pub struct TopicDeleted {
    pub name: String,
}
