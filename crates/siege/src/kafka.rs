use std::future::Future;
use std::pin::Pin;

use siege_kernel::KafkaProperties;

use crate::SiegeError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq)]
pub struct TopicMeta {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: KafkaProperties,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: KafkaProperties,
}

pub trait KafkaBackend: Send + Sync + 'static {
    fn list_topics(&self) -> BoxFuture<'_, Result<Vec<TopicMeta>, SiegeError>>;
    fn get_topic(&self, name: &str) -> BoxFuture<'_, Result<TopicDetail, SiegeError>>;
    fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> BoxFuture<'_, Result<(), SiegeError>>;
    fn delete_topic(&self, name: &str) -> BoxFuture<'_, Result<(), SiegeError>>;
    fn update_topic_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> BoxFuture<'_, Result<(), SiegeError>>;
    fn create_partitions(
        &self,
        name: &str,
        total: usize,
    ) -> BoxFuture<'_, Result<(), SiegeError>>;
    fn producer(&self) -> Box<dyn KafkaProducer>;
}

pub trait KafkaProducer: Send + Sync {
    fn send<'a>(
        &'a self,
        topic: &'a str,
        key: Option<&'a [u8]>,
        payload: &'a [u8],
    ) -> BoxFuture<'a, Result<(), SiegeError>>;
}
