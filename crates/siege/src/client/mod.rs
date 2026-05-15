mod chaos;
mod seed;
mod topics;

use crate::SiegeContext;

pub use chaos::Chaos;
pub use seed::Seed;
pub use topics::Topics;

pub struct Client<C: SiegeContext> {
    ctx: C,
}

impl<C: SiegeContext> Client<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }

    pub fn chaos(&self) -> Chaos<'_, C> {
        Chaos::new(&self.ctx)
    }

    pub fn seeder(&self) -> Seed<'_, C> {
        Seed::new(&self.ctx)
    }

    pub fn topics(&self) -> Topics<'_, C> {
        Topics::new(&self.ctx)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{
        EventEmitter, KafkaProperties, MockKafkaBackend, NoopChaos, NoopSchemaRegistry,
        NoopSeeder, SiegeContext, SiegeError,
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
                DomainEvent::ChaosTopicDeleted(_) => "chaos_topic_deleted",
                DomainEvent::ChaosRetentionLowered(_) => "chaos_retention_lowered",
                DomainEvent::ChaosCleanupPolicyFlipped(_) => "chaos_cleanup_policy_flipped",
                DomainEvent::ChaosPartitionsIncreased(_) => "chaos_partitions_increased",
                DomainEvent::ChaosPoisonPillsSent(_) => "chaos_poison_pills_sent",
                DomainEvent::ChaosSchemaBreakSent(_) => "chaos_schema_break_sent",
                DomainEvent::TopicsSeeded(_) => "topics_seeded",
            };
            self.events.lock().unwrap().push(name.to_owned());
        }
    }

    struct TestCtx {
        kafka: MockKafkaBackend,
        events: RecordingEmitter,
        chaos: NoopChaos,
        seeder: NoopSeeder,
    }

    impl SiegeContext for TestCtx {
        type Kafka = MockKafkaBackend;
        type Events = RecordingEmitter;
        type Chaos = NoopChaos;
        type Seeder = NoopSeeder;
        type SchemaRegistry = NoopSchemaRegistry;

        fn kafka(&self) -> &MockKafkaBackend { &self.kafka }
        fn events(&self) -> &RecordingEmitter { &self.events }
        fn chaos(&self) -> &NoopChaos { &self.chaos }
        fn seeder(&self) -> &NoopSeeder { &self.seeder }
        fn schema_registry(&self) -> Option<&NoopSchemaRegistry> { None }
    }

    fn test_client(topics: Vec<TopicDetail>) -> Client<TestCtx> {
        Client::new(TestCtx {
            kafka: MockKafkaBackend::with_topics(topics),
            events: RecordingEmitter::default(),
            chaos: NoopChaos,
            seeder: NoopSeeder,
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
        assert_eq!(topic.config.get("retention.ms").unwrap(), "1000");
    }
}
