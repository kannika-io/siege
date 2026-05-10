mod topics;

use crate::SiegeContext;

pub use topics::Topics;

pub struct Client<C: SiegeContext> {
    ctx: C,
}

impl<C: SiegeContext> Client<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }

    pub fn topics(&self) -> Topics<'_, C> {
        Topics::new(&self.ctx)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::kafka::KafkaBackend;
    use crate::{
        EventEmitter, KafkaProperties, SiegeContext, SiegeError, Topic, TopicDetail,
    };
    use crate::event::DomainEvent;

    use super::*;

    #[derive(Clone, Default)]
    struct FakeKafka {
        topics: Arc<Mutex<Vec<TopicDetail>>>,
    }

    impl KafkaBackend for FakeKafka {
        fn list_topics(
            &self,
        ) -> impl Future<Output = Result<Vec<Topic>, SiegeError>> + Send {
            let topics = self
                .topics
                .lock()
                .unwrap()
                .iter()
                .map(|d| Topic {
                    name: d.name.clone(),
                    partitions: d.partitions,
                    replication_factor: d.replication_factor,
                })
                .collect();
            async move { Ok(topics) }
        }

        fn get_topic(
            &self,
            name: &str,
        ) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send {
            let result = self
                .topics
                .lock()
                .unwrap()
                .iter()
                .find(|t| t.name == name)
                .cloned()
                .ok_or_else(|| SiegeError::TopicNotFound(name.to_owned()));
            async move { result }
        }

        fn create_topic(
            &self,
            name: &str,
            partitions: i32,
            replication_factor: i32,
            config: KafkaProperties,
        ) -> impl Future<Output = Result<(), SiegeError>> + Send {
            let mut topics = self.topics.lock().unwrap();
            let result = if topics.iter().any(|t| t.name == name) {
                Err(SiegeError::TopicAlreadyExists(name.to_owned()))
            } else {
                topics.push(TopicDetail {
                    name: name.to_owned(),
                    partitions,
                    replication_factor,
                    config,
                });
                Ok(())
            };
            async move { result }
        }

        fn delete_topic(
            &self,
            name: &str,
        ) -> impl Future<Output = Result<(), SiegeError>> + Send {
            let mut topics = self.topics.lock().unwrap();
            let len = topics.len();
            topics.retain(|t| t.name != name);
            let result = if topics.len() < len {
                Ok(())
            } else {
                Err(SiegeError::TopicNotFound(name.to_owned()))
            };
            async move { result }
        }

        fn update_topic_config(
            &self,
            name: &str,
            config: KafkaProperties,
        ) -> impl Future<Output = Result<(), SiegeError>> + Send {
            let mut topics = self.topics.lock().unwrap();
            let result = match topics.iter_mut().find(|t| t.name == name) {
                Some(t) => {
                    t.config.extend(config);
                    Ok(())
                }
                None => Err(SiegeError::TopicNotFound(name.to_owned())),
            };
            async move { result }
        }
    }

    #[derive(Default)]
    struct RecordingEmitter {
        events: Mutex<Vec<String>>,
    }

    impl EventEmitter for RecordingEmitter {
        fn emit(&self, event: &DomainEvent) {
            let name = match event {
                DomainEvent::TopicCreated { .. } => "topic_created",
                DomainEvent::TopicDeleted { .. } => "topic_deleted",
            };
            self.events.lock().unwrap().push(name.to_owned());
        }
    }

    struct TestCtx {
        kafka: FakeKafka,
        events: RecordingEmitter,
    }

    impl SiegeContext for TestCtx {
        type Kafka = FakeKafka;
        type Events = RecordingEmitter;

        fn kafka(&self) -> &FakeKafka {
            &self.kafka
        }
        fn events(&self) -> &RecordingEmitter {
            &self.events
        }
    }

    fn test_client(topics: Vec<TopicDetail>) -> Client<TestCtx> {
        Client::new(TestCtx {
            kafka: FakeKafka {
                topics: Arc::new(Mutex::new(topics)),
            },
            events: RecordingEmitter::default(),
        })
    }

    fn sample(name: &str) -> TopicDetail {
        TopicDetail {
            name: name.into(),
            partitions: 3,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }
    }

    #[tokio::test]
    async fn list_topics() {
        let client = test_client(vec![sample("a"), sample("b")]);
        let topics = client.topics().list().await.unwrap();
        assert_eq!(topics.len(), 2);
    }

    #[tokio::test]
    async fn get_topic() {
        let client = test_client(vec![sample("my-topic")]);
        let detail = client.topics().get("my-topic").await.unwrap();
        assert_eq!(detail.name, "my-topic");
    }

    #[tokio::test]
    async fn get_topic_not_found() {
        let client = test_client(vec![]);
        let err = client.topics().get("nope").await.unwrap_err();
        assert!(matches!(err, SiegeError::TopicNotFound(_)));
    }

    #[tokio::test]
    async fn create_topic_emits_event() {
        let client = test_client(vec![]);
        let created = client.topics().create("new", 6, 3, KafkaProperties::new()).await.unwrap();
        assert_eq!(created.topic.name, "new");
        assert_eq!(created.topic.partitions, 6);

        let events = client.ctx.events.events.lock().unwrap();
        assert_eq!(events.as_slice(), &["topic_created"]);
    }

    #[tokio::test]
    async fn delete_topic_emits_event() {
        let client = test_client(vec![sample("doomed")]);
        let deleted = client.topics().delete("doomed").await.unwrap();
        assert_eq!(deleted.name, "doomed");

        let events = client.ctx.events.events.lock().unwrap();
        assert_eq!(events.as_slice(), &["topic_deleted"]);
    }

    #[tokio::test]
    async fn update_config() {
        let client = test_client(vec![sample("t")]);
        let config: KafkaProperties =
            std::collections::HashMap::from([("retention.ms".into(), "1000".into())]).into();
        client.topics().update_config("t", config).await.unwrap();
        let detail = client.topics().get("t").await.unwrap();
        assert_eq!(detail.config.get("retention.ms").unwrap(), "1000");
    }
}
