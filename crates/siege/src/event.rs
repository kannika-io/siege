use crate::kafka::TopicDetail;

pub struct TopicCreatedEvent {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: siege_kernel::KafkaProperties,
}

pub struct TopicDeletedEvent {
    pub name: String,
}

pub struct ChaosTopicDeletedEvent {
    pub topic: String,
}

pub struct ChaosRetentionLoweredEvent {
    pub detail: TopicDetail,
}

pub struct ChaosCleanupPolicyFlippedEvent {
    pub detail: TopicDetail,
}

pub struct ChaosPartitionsIncreasedEvent {
    pub detail: TopicDetail,
}

pub struct ChaosPoisonPillsSentEvent {
    pub topic: String,
    pub count: u32,
}

pub struct ChaosSchemaBreakSentEvent {
    pub topic: String,
    pub count: u32,
}

pub struct SeedProgressEvent {
    pub topic: String,
    pub topic_index: u32,
    pub total_topics: u32,
    pub records_generated: u32,
    pub total_records: u32,
}

pub struct TopicsSeededEvent {
    pub topics: Vec<TopicDetail>,
}

pub enum DomainEvent {
    TopicCreated(TopicCreatedEvent),
    TopicDeleted(TopicDeletedEvent),
    ChaosTopicDeleted(ChaosTopicDeletedEvent),
    ChaosRetentionLowered(ChaosRetentionLoweredEvent),
    ChaosCleanupPolicyFlipped(ChaosCleanupPolicyFlippedEvent),
    ChaosPartitionsIncreased(ChaosPartitionsIncreasedEvent),
    ChaosPoisonPillsSent(ChaosPoisonPillsSentEvent),
    ChaosSchemaBreakSent(ChaosSchemaBreakSentEvent),
    SeedProgress(SeedProgressEvent),
    TopicsSeeded(TopicsSeededEvent),
}

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, event: &DomainEvent);
}
