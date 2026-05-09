use siege_kernel::KafkaProperties;

use crate::kafka::KafkaBackend;
use crate::{SiegeError, Topic, TopicCreated, TopicDeleted, TopicDetail};

pub async fn list_topics(kafka: &impl KafkaBackend) -> Result<Vec<Topic>, SiegeError> {
    kafka.list_topics().await
}

pub async fn get_topic(kafka: &impl KafkaBackend, name: &str) -> Result<TopicDetail, SiegeError> {
    kafka.get_topic(name).await
}

pub async fn create_topic(
    kafka: &impl KafkaBackend,
    name: &str,
    partitions: i32,
    replication_factor: i32,
) -> Result<TopicCreated, SiegeError> {
    kafka.create_topic(name, partitions, replication_factor).await?;
    Ok(TopicCreated {
        topic: Topic {
            name: name.to_owned(),
            partitions,
            replication_factor,
        },
    })
}

pub async fn delete_topic(
    kafka: &impl KafkaBackend,
    name: &str,
) -> Result<TopicDeleted, SiegeError> {
    kafka.delete_topic(name).await?;
    Ok(TopicDeleted {
        name: name.to_owned(),
    })
}

pub async fn update_topic_config(
    kafka: &impl KafkaBackend,
    name: &str,
    config: KafkaProperties,
) -> Result<(), SiegeError> {
    kafka.update_topic_config(name, config).await
}
