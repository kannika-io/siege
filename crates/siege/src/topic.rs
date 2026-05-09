use serde::{Deserialize, Serialize};
use siege_kernel::KafkaProperties;

use crate::DomainEvent;

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

impl DomainEvent for TopicCreated {
    fn event_name(&self) -> &'static str {
        "topic_created"
    }
}

pub struct TopicDeleted {
    pub name: String,
}

impl DomainEvent for TopicDeleted {
    fn event_name(&self) -> &'static str {
        "topic_deleted"
    }
}
