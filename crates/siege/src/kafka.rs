use std::future::Future;

use siege_kernel::KafkaProperties;

use crate::SiegeError;

#[derive(Debug, Clone, PartialEq)]
pub struct TopicMeta {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: KafkaProperties,
}

pub trait KafkaBackend: Send + Sync + 'static {
    fn list_topics(&self) -> impl Future<Output = Result<Vec<TopicMeta>, SiegeError>> + Send;
    fn get_topic(&self, name: &str) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send;
    fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send;
    fn delete_topic(&self, name: &str) -> impl Future<Output = Result<(), SiegeError>> + Send;
    fn update_topic_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send;
}
