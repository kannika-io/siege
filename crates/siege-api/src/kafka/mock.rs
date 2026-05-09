use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use siege_api_spec::*;

use super::backend::KafkaBackend;

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
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, SiegeError>> + Send {
        let topics: Vec<Topic> = self
            .topics
            .lock()
            .unwrap()
            .values()
            .map(|d| Topic {
                name: d.name.clone(),
                partitions: d.partitions,
                replication_factor: d.replication_factor,
            })
            .collect();
        async move { Ok(topics) }
    }

    fn get_topic(&self, name: &str) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send {
        let result = self
            .topics
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| SiegeError::TopicNotFound(name.to_owned()));
        async move { result }
    }

    fn create_topic(
        &self,
        req: CreateTopicRequest,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            if topics.contains_key(&req.name) {
                Err(SiegeError::TopicAlreadyExists(req.name.clone()))
            } else {
                topics.insert(
                    req.name.clone(),
                    TopicDetail {
                        name: req.name,
                        partitions: req.partitions,
                        replication_factor: req.replication_factor,
                        config: KafkaProperties::new(),
                    },
                );
                Ok(())
            }
        };
        async move { result }
    }

    fn delete_topic(&self, name: &str) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            if topics.remove(name).is_some() {
                Ok(())
            } else {
                Err(SiegeError::TopicNotFound(name.to_owned()))
            }
        };
        async move { result }
    }

    fn update_topic_config(
        &self,
        name: &str,
        config: TopicConfigUpdate,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let result = {
            let mut topics = self.topics.lock().unwrap();
            match topics.get_mut(name) {
                Some(detail) => {
                    detail.config.extend(config.config);

                    Ok(())
                }
                None => Err(SiegeError::TopicNotFound(name.to_owned())),
            }
        };
        async move { result }
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
            .create_topic(CreateTopicRequest {
                name: "new".into(),
                partitions: 6,
                replication_factor: 3,
            })
            .await
            .unwrap();
        let detail = backend.get_topic("new").await.unwrap();
        assert_eq!(detail.partitions, 6);
    }

    #[tokio::test]
    async fn create_topic_already_exists() {
        let backend = MockKafkaBackend::with_topics(vec![sample_detail("existing")]);
        let err = backend
            .create_topic(CreateTopicRequest {
                name: "existing".into(),
                partitions: 1,
                replication_factor: 1,
            })
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
        backend
            .update_topic_config(
                "t",
                TopicConfigUpdate {
                    config: HashMap::from([("retention.ms".into(), "1000".into())]).into(),
                },
            )
            .await
            .unwrap();
        let detail = backend.get_topic("t").await.unwrap();
        assert_eq!(detail.config.get("retention.ms").unwrap(), "1000");
    }
}
