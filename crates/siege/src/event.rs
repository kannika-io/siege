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

pub struct ChaosRetentionZeroedEvent {
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

pub struct TopicsSeededEvent {
    pub topics: Vec<TopicDetail>,
}

pub enum DomainEvent {
    TopicCreated(TopicCreatedEvent),
    TopicDeleted(TopicDeletedEvent),
    ChaosTopicDeleted(ChaosTopicDeletedEvent),
    ChaosRetentionZeroed(ChaosRetentionZeroedEvent),
    ChaosCleanupPolicyFlipped(ChaosCleanupPolicyFlippedEvent),
    ChaosPartitionsIncreased(ChaosPartitionsIncreasedEvent),
    ChaosPoisonPillsSent(ChaosPoisonPillsSentEvent),
    ChaosSchemaBreakSent(ChaosSchemaBreakSentEvent),
    TopicsSeeded(TopicsSeededEvent),
}

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, event: &DomainEvent);
}
