use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::chaos::ChaosBackend;
use crate::kafka::{BoxFuture, KafkaBackend, KafkaProducer, TopicDetail, TopicMeta};
use crate::schema_registry::{SchemaId, SchemaRegistryBackend};
use crate::seed::{SeedBackend, SeedResult};
use crate::{KafkaProperties, SiegeError};

pub struct NoopChaos;

impl ChaosBackend for NoopChaos {
    type Error = SiegeError;

    async fn get_topic(&self, name: &str) -> Result<TopicDetail, SiegeError> {
        Err(SiegeError::TopicNotFound(name.to_owned()))
    }
    async fn delete_topic(&self, _topic: &str) -> Result<(), SiegeError> { Ok(()) }
    async fn low_retention(&self, _topic: &str) -> Result<(), SiegeError> { Ok(()) }
    async fn flip_cleanup_policy(&self, _topic: &str) -> Result<(), SiegeError> { Ok(()) }
    async fn increase_partitions(&self, _topic: &str, _extra: i32) -> Result<(), SiegeError> { Ok(()) }
    async fn poison_pills(&self, _topic: &str, _count: u32) -> Result<(), SiegeError> { Ok(()) }
    async fn schema_break(&self, _topic: &str, _count: u32) -> Result<(), SiegeError> { Ok(()) }
}

pub struct NoopSeeder;

impl SeedBackend for NoopSeeder {
    async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        Ok(SeedResult { created: vec![], skipped: vec![] })
    }
}

pub struct NoopSchemaRegistry;

impl SchemaRegistryBackend for NoopSchemaRegistry {
    fn register_schema(
        &self,
        _subject: &str,
        _schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>> {
        Box::pin(async { Ok(SchemaId(1)) })
    }

    fn delete_subject(
        &self,
        _subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Clone, Default)]
pub struct MockKafkaBackend {
    topics: Arc<Mutex<HashMap<String, TopicDetail>>>,
}

impl MockKafkaBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_topics(topics: Vec<TopicDetail>) -> Self {
        let map = topics.into_iter().map(|t| (t.name.clone(), t)).collect();
        Self {
            topics: Arc::new(Mutex::new(map)),
        }
    }
}

impl KafkaBackend for MockKafkaBackend {
    fn list_topics(&self) -> BoxFuture<'_, Result<Vec<TopicMeta>, SiegeError>> {
        let topics: Vec<TopicMeta> = self
            .topics
            .lock()
            .unwrap()
            .values()
            .map(|d| TopicMeta {
                name: d.name.clone(),
                partitions: d.partitions,
                replication_factor: d.replication_factor,
                config: d.config.clone(),
            })
            .collect();
        Box::pin(async move { Ok(topics) })
    }

    fn get_topic(&self, name: &str) -> BoxFuture<'_, Result<TopicDetail, SiegeError>> {
        let result = self
            .topics
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| SiegeError::TopicNotFound(name.to_owned()));
        Box::pin(async move { result })
    }

    fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        let name = name.to_owned();
        let result = {
            let mut topics = self.topics.lock().unwrap();
            if topics.contains_key(&name) {
                Err(SiegeError::TopicAlreadyExists(name.clone()))
            } else {
                topics.insert(
                    name.clone(),
                    TopicDetail {
                        name,
                        partitions,
                        replication_factor,
                        config,
                    },
                );
                Ok(())
            }
        };
        Box::pin(async move { result })
    }

    fn delete_topic(&self, name: &str) -> BoxFuture<'_, Result<(), SiegeError>> {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            if topics.remove(name).is_some() {
                Ok(())
            } else {
                Err(SiegeError::TopicNotFound(name.to_owned()))
            }
        };
        Box::pin(async move { result })
    }

    fn update_topic_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            match topics.get_mut(name) {
                Some(detail) => {
                    detail.config.extend(config);
                    Ok(())
                }
                None => Err(SiegeError::TopicNotFound(name.to_owned())),
            }
        };
        Box::pin(async move { result })
    }

    fn create_partitions(
        &self,
        name: &str,
        total: usize,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            match topics.get_mut(name) {
                Some(detail) => {
                    detail.partitions = total as i32;
                    Ok(())
                }
                None => Err(SiegeError::TopicNotFound(name.to_owned())),
            }
        };
        Box::pin(async move { result })
    }

    fn producer(&self) -> Box<dyn KafkaProducer> {
        struct NoopProducer;
        impl KafkaProducer for NoopProducer {
            fn send<'a>(&'a self, _topic: &'a str, _key: Option<&'a [u8]>, _payload: &'a [u8]) -> BoxFuture<'a, Result<(), SiegeError>> {
                Box::pin(async { Ok(()) })
            }
        }
        Box::new(NoopProducer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_detail(name: &str) -> TopicDetail {
        TopicDetail {
            name: name.into(),
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }
    }

    #[tokio::test]
    async fn list_topics_returns_all() {
        let backend = MockKafkaBackend::with_topics(vec![
            sample_detail("topic-a"),
            sample_detail("topic-b"),
        ]);
        let topics = backend.list_topics().await.unwrap();
        assert_eq!(topics.len(), 2);
    }

    #[tokio::test]
    async fn get_topic_found() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("topic-a")]);
        let detail = backend.get_topic("topic-a").await.unwrap();
        assert_eq!(detail.name, "topic-a");
        assert_eq!(detail.partitions, 3);
    }

    #[tokio::test]
    async fn get_topic_not_found() {
        let backend = MockKafkaBackend::new();
        let err = backend.get_topic("nope").await.unwrap_err();
        assert!(matches!(err, SiegeError::TopicNotFound(_)));
    }

    #[tokio::test]
    async fn create_topic_success() {
        let backend = MockKafkaBackend::new();
        backend
            .create_topic("new", 6, 3, KafkaProperties::new())
            .await
            .unwrap();
        let detail = backend.get_topic("new").await.unwrap();
        assert_eq!(detail.partitions, 6);
    }

    #[tokio::test]
    async fn create_topic_already_exists() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("existing")]);
        let err = backend
            .create_topic("existing", 1, 1, KafkaProperties::new())
            .await
            .unwrap_err();
        assert!(matches!(err, SiegeError::TopicAlreadyExists(_)));
    }

    #[tokio::test]
    async fn delete_topic_success() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("doomed")]);
        backend.delete_topic("doomed").await.unwrap();
        assert!(backend.get_topic("doomed").await.is_err());
    }

    #[tokio::test]
    async fn update_config_merges() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("t")]);
        let config: KafkaProperties =
            HashMap::from([("retention.ms".into(), "1000".into())]).into();
        backend.update_topic_config("t", config).await.unwrap();
        let detail = backend.get_topic("t").await.unwrap();
        assert_eq!(detail.config.get("retention.ms").unwrap(), "1000");
    }
}
