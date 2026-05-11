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
    use std::sync::Mutex;

    use crate::{
        EventEmitter, KafkaProperties, MockKafkaBackend, SiegeContext, SiegeError,
    };
    use crate::event::DomainEvent;
    use crate::kafka::TopicDetail;

    use super::*;

    #[derive(Default)]
    struct RecordingEmitter {
        events: Mutex<Vec<String>>,
    }

    impl EventEmitter for RecordingEmitter {
        fn emit(&self, event: &DomainEvent) {
            let name = match event {
                DomainEvent::TopicCreated(_) => "topic_created",
                DomainEvent::TopicDeleted(_) => "topic_deleted",
            };
            self.events.lock().unwrap().push(name.to_owned());
        }
    }

    struct TestCtx {
        kafka: MockKafkaBackend,
        events: RecordingEmitter,
    }

    impl SiegeContext for TestCtx {
        type Kafka = MockKafkaBackend;
        type Events = RecordingEmitter;

        fn kafka(&self) -> &MockKafkaBackend {
            &self.kafka
        }
        fn events(&self) -> &RecordingEmitter {
            &self.events
        }
    }

    fn test_client(topics: Vec<TopicDetail>) -> Client<TestCtx> {
        Client::new(TestCtx {
            kafka: MockKafkaBackend::with_topics(topics),
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
        let topic = client.topics().get("my-topic").await.unwrap();
        assert_eq!(topic.name, "my-topic");
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
        let topic = client
            .topics()
            .create("new", 6, 3, KafkaProperties::new())
            .await
            .unwrap();
        assert_eq!(topic.name, "new");
        assert_eq!(topic.partitions, 6);

        let events = client.ctx.events.events.lock().unwrap();
        assert_eq!(events.as_slice(), &["topic_created"]);
    }

    #[tokio::test]
    async fn delete_topic_emits_event() {
        let client = test_client(vec![sample("doomed")]);
        client.topics().delete("doomed").await.unwrap();

        let events = client.ctx.events.events.lock().unwrap();
        assert_eq!(events.as_slice(), &["topic_deleted"]);
    }

    #[tokio::test]
    async fn get_then_delete() {
        let client = test_client(vec![sample("doomed")]);
        let topic = client.topics().get("doomed").await.unwrap();
        topic.delete().await.unwrap();

        let events = client.ctx.events.events.lock().unwrap();
        assert_eq!(events.as_slice(), &["topic_deleted"]);
    }

    #[tokio::test]
    async fn topic_config() {
        let client = test_client(vec![sample("t")]);
        let config: KafkaProperties =
            std::collections::HashMap::from([("retention.ms".into(), "1000".into())]).into();
        client
            .topics()
            .update_config("t", config)
            .await
            .unwrap();
        let topic = client.topics().get("t").await.unwrap();
        let config = topic.config().await.unwrap();
        assert_eq!(config.get("retention.ms").unwrap(), "1000");
    }
}
